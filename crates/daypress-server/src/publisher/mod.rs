//! # Static Edge Publisher & Deployment Ledger (`crates/daypress-server/src/publisher/`)
//!
//! Compiles the local Daypress CMS (`localhost:8090`) into a self-contained static build
//! via in-memory Axum SSR (`OptionalAuthUser(None)`), generates `_redirects`, `sitemap.xml`,
//! `robots.txt`, `feed/index.xml`, `404.html`, and client-side `search-index.json`, and
//! deploys to Cloudflare Pages, Netlify, or a local folder / `.zip` archive.

pub mod archive;
pub mod providers;

use crate::db::Database;
use crate::errors::AppError;
use archive::{build_zip_archive, read_zip_archive, sha256_hex};
use axum::body::{to_bytes, Body};
use axum::http::Request;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const MASKED_SECRET: &str = "********";

/// Stored publishing & hosting settings in SQLite (`options` table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishSettings {
    pub provider: String,
    pub production_url: String,
    pub cf_account_id: String,
    pub cf_api_token: String,
    pub cf_project_name: String,
    pub cf_custom_domain: String,
    pub netlify_token: String,
    pub netlify_site_id: String,
    pub netlify_custom_domain: String,
    pub local_export_dir: String,
    pub static_search_enabled: bool,
    pub static_redirects_enabled: bool,
    pub static_form_mode: String,
    pub static_form_webhook_url: String,
    pub neutralize_admin_links: bool,
    pub has_cf_token: bool,
    pub has_netlify_token: bool,
}

impl Default for PublishSettings {
    fn default() -> Self {
        Self {
            provider: "zip".to_string(),
            production_url: String::new(),
            cf_account_id: String::new(),
            cf_api_token: String::new(),
            cf_project_name: String::new(),
            cf_custom_domain: String::new(),
            netlify_token: String::new(),
            netlify_site_id: String::new(),
            netlify_custom_domain: String::new(),
            local_export_dir: "data/static-export".to_string(),
            static_search_enabled: true,
            static_redirects_enabled: true,
            static_form_mode: "auto".to_string(),
            static_form_webhook_url: String::new(),
            neutralize_admin_links: true,
            has_cf_token: false,
            has_netlify_token: false,
        }
    }
}

impl PublishSettings {
    /// Returns a copy with API tokens masked (`********`) for safe client/JSON responses.
    pub fn redacted(&self) -> Self {
        let mut copy = self.clone();
        copy.has_cf_token = !self.cf_api_token.trim().is_empty();
        copy.has_netlify_token = !self.netlify_token.trim().is_empty();
        if copy.has_cf_token {
            copy.cf_api_token = MASKED_SECRET.to_string();
        }
        if copy.has_netlify_token {
            copy.netlify_token = MASKED_SECRET.to_string();
        }
        copy
    }

    /// Resolves the canonical public origin URL for rewriting local `http://localhost:8090` URLs.
    pub fn resolved_target_origin(&self) -> String {
        let prod = self.production_url.trim().trim_end_matches('/');
        if !prod.is_empty() {
            if prod.starts_with("http://") || prod.starts_with("https://") {
                return prod.to_string();
            }
            return format!("https://{prod}");
        }
        match self.provider.as_str() {
            "cloudflare" => {
                let dom = self.cf_custom_domain.trim().trim_end_matches('/');
                if !dom.is_empty() {
                    let clean = dom
                        .trim_start_matches("https://")
                        .trim_start_matches("http://");
                    format!("https://{clean}")
                } else if !self.cf_project_name.trim().is_empty() {
                    format!("https://{}.pages.dev", self.cf_project_name.trim())
                } else {
                    String::new()
                }
            }
            "netlify" => {
                let dom = self.netlify_custom_domain.trim().trim_end_matches('/');
                if !dom.is_empty() {
                    let clean = dom
                        .trim_start_matches("https://")
                        .trim_start_matches("http://");
                    format!("https://{clean}")
                } else if !self.netlify_site_id.trim().is_empty() {
                    format!("https://{}.netlify.app", self.netlify_site_id.trim())
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }
}

/// Request payload for `POST /wp-json/daypress/v1/publish/settings`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PublishSettingsUpdateRequest {
    pub provider: Option<String>,
    pub production_url: Option<String>,
    pub cf_account_id: Option<String>,
    pub cf_api_token: Option<String>,
    pub cf_project_name: Option<String>,
    pub cf_custom_domain: Option<String>,
    pub netlify_token: Option<String>,
    pub netlify_site_id: Option<String>,
    pub netlify_custom_domain: Option<String>,
    pub local_export_dir: Option<String>,
    pub static_search_enabled: Option<bool>,
    pub static_redirects_enabled: Option<bool>,
    pub static_form_mode: Option<String>,
    pub static_form_webhook_url: Option<String>,
    pub neutralize_admin_links: Option<bool>,
}

/// Request payload for `POST /wp-json/daypress/v1/publish/run`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PublishRunRequest {
    #[serde(default)]
    pub commit_message: String,
    #[serde(default)]
    pub provider_override: Option<String>,
}

/// Request payload for `POST /wp-json/daypress/v1/publish/domain/configure` & `/verify`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DomainActionRequest {
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfigResult {
    pub domain: String,
    pub provider: String,
    pub auto_dns_provisioned: bool,
    pub record_type: String,
    pub record_name: String,
    pub record_target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerifyResult {
    pub domain: String,
    pub dns_resolved: bool,
    pub resolved_ips: Vec<String>,
    pub ssl_active: bool,
    pub status_message: String,
}

/// A single local modification queued for the next publish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpublishedChangeItem {
    pub kind: String,
    pub title: String,
    pub slug_or_path: String,
    pub modified_at: String,
    pub change_type: String,
}

/// A pre-flight audit note or warning shown before publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightWarning {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub source: String,
}

/// A row in `daypress_deployments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: i64,
    pub provider: String,
    pub target_url: String,
    pub deployment_external_id: Option<String>,
    pub status: String,
    pub commit_message: String,
    pub pages_count: i64,
    pub assets_count: i64,
    pub total_bytes: i64,
    pub bundle_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub duration_ms: i64,
}

/// Full status payload for `/d-admin/publish` and `GET /wp-json/daypress/v1/publish/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishStatusResponse {
    pub settings: PublishSettings,
    pub unpublished_changes: Vec<UnpublishedChangeItem>,
    pub preflight_warnings: Vec<PreflightWarning>,
    pub deployments: Vec<DeploymentRecord>,
    pub last_live_deployment: Option<DeploymentRecord>,
}

/// Manifest stored in `daypress_deployments.manifest_json` for diffing & delta uploads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentManifest {
    pub files: BTreeMap<String, String>,
    pub post_timestamps: BTreeMap<String, String>,
    pub template_timestamps: BTreeMap<String, String>,
    pub media_ids: Vec<i64>,
}

/// Output of the in-memory Static Site Generator before uploading/archiving.
pub struct BuiltStaticSite {
    pub files: BTreeMap<String, Vec<u8>>,
    pub pages_count: usize,
    pub assets_count: usize,
    pub total_bytes: usize,
    pub manifest: DeploymentManifest,
}

const CLIENT_STATIC_SEARCH_JS: &str = r#"/*! Daypress Static Edge Client-Side Search */
(function () {
  var indexCache = null;
  function loadIndex() {
    if (indexCache) return Promise.resolve(indexCache);
    return fetch('/search-index.json')
      .then(function (r) { return r.ok ? r.json() : []; })
      .then(function (data) { indexCache = Array.isArray(data) ? data : []; return indexCache; })
      .catch(function () { return []; });
  }
  function escapeHtml(s) {
    return String(s || '').replace(/[&<>"']/g, function (c) {
      return ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c];
    });
  }
  function renderStaticSearchResults(query) {
    var clean = String(query || '').trim();
    if (!clean) return;
    loadIndex().then(function (items) {
      var q = clean.toLowerCase();
      var matches = items.filter(function (it) {
        return (it.title && it.title.toLowerCase().indexOf(q) !== -1) ||
               (it.excerpt && it.excerpt.toLowerCase().indexOf(q) !== -1) ||
               (it.slug && it.slug.toLowerCase().indexOf(q) !== -1);
      });
      var container = document.querySelector('main, .site-main, .entry-content, body');
      if (!container) return;
      var modal = document.getElementById('daypress-static-search-results');
      if (!modal) {
        modal = document.createElement('section');
        modal.id = 'daypress-static-search-results';
        modal.style.cssText = 'max-width:820px;margin:32px auto;padding:24px;border:1px solid rgba(128,128,128,0.25);border-radius:8px;';
        container.prepend(modal);
      }
      var listHtml = matches.length
        ? matches.map(function (m) {
            return '<article style="padding:14px 0;border-bottom:1px solid rgba(128,128,128,0.15);">' +
              '<h3 style="margin:0 0 6px;font-size:18px;"><a href="' + escapeHtml(m.url) + '">' + escapeHtml(m.title) + '</a></h3>' +
              '<p style="margin:0;font-size:14px;opacity:0.8;">' + escapeHtml(m.excerpt) + '</p>' +
            '</article>';
          }).join('')
        : '<p>No matching pages or posts found.</p>';
      modal.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">' +
        '<strong>Search results for “' + escapeHtml(clean) + '” (' + matches.length + ')</strong>' +
        '<button type="button" onclick="this.closest(\'#daypress-static-search-results\').remove()" style="background:none;border:none;cursor:pointer;font-size:18px;">&times;</button>' +
        '</div>' + listHtml;
    });
  }
  document.addEventListener('DOMContentLoaded', function () {
    var params = new URLSearchParams(window.location.search);
    if (params.get('s')) {
      renderStaticSearchResults(params.get('s'));
    }
    document.querySelectorAll('form').forEach(function (form) {
      var sInput = form.querySelector('input[name="s"]');
      if (sInput) {
        form.addEventListener('submit', function (e) {
          e.preventDefault();
          renderStaticSearchResults(sInput.value);
        });
      }
      var webhook = form.getAttribute('data-static-webhook');
      if (webhook) {
        form.addEventListener('submit', function (e) {
          e.preventDefault();
          var fd = new FormData(form);
          var payload = {};
          fd.forEach(function (v, k) { payload[k] = v; });
          fetch(webhook, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
            body: JSON.stringify(payload)
          }).then(function () {
            var msg = form.querySelector('.daypress-form-feedback') || document.createElement('div');
            msg.className = 'daypress-form-feedback';
            msg.style.cssText = 'margin-top:12px;padding:10px 14px;border-radius:6px;background:rgba(34,197,94,0.12);color:#15803d;font-weight:600;';
            msg.textContent = 'Thank you! Your message has been sent.';
            if (!msg.parentNode) form.appendChild(msg);
            form.reset();
          });
        });
      }
    });
  });
})();
"#;

impl Database {
    /// Reads the current `PublishSettings` from the `options` table.
    pub fn get_publish_settings(&self) -> PublishSettings {
        let mut s = PublishSettings::default();
        let get = |k: &str| self.get_option(k).ok().flatten().unwrap_or_default();

        let provider = get("daypress_publish_provider");
        if !provider.trim().is_empty() {
            s.provider = provider.trim().to_ascii_lowercase();
        }
        s.production_url = get("daypress_publish_production_url");
        s.cf_account_id = get("daypress_cf_account_id");
        s.cf_api_token = get("daypress_cf_api_token");
        s.cf_project_name = get("daypress_cf_project_name");
        s.cf_custom_domain = get("daypress_cf_custom_domain");
        s.netlify_token = get("daypress_netlify_token");
        s.netlify_site_id = get("daypress_netlify_site_id");
        s.netlify_custom_domain = get("daypress_netlify_custom_domain");
        let export_dir = get("daypress_publish_export_dir");
        if !export_dir.trim().is_empty() {
            s.local_export_dir = export_dir;
        }
        let search_opt = get("daypress_static_search_enabled");
        if !search_opt.is_empty() {
            s.static_search_enabled =
                search_opt != "0" && !search_opt.eq_ignore_ascii_case("false");
        }
        let redir_opt = get("daypress_static_redirects_enabled");
        if !redir_opt.is_empty() {
            s.static_redirects_enabled =
                redir_opt != "0" && !redir_opt.eq_ignore_ascii_case("false");
        }
        let form_mode = get("daypress_static_form_mode");
        if !form_mode.trim().is_empty() {
            s.static_form_mode = form_mode;
        }
        s.static_form_webhook_url = get("daypress_static_form_webhook_url");
        let neut_opt = get("daypress_neutralize_admin_links");
        if !neut_opt.is_empty() {
            s.neutralize_admin_links = neut_opt != "0" && !neut_opt.eq_ignore_ascii_case("false");
        }
        s.has_cf_token = !s.cf_api_token.trim().is_empty();
        s.has_netlify_token = !s.netlify_token.trim().is_empty();
        s
    }

    /// Persists updated `PublishSettings` into the `options` table.
    pub fn save_publish_settings(
        &self,
        req: &PublishSettingsUpdateRequest,
    ) -> Result<PublishSettings, AppError> {
        let current = self.get_publish_settings();

        let provider = req
            .provider
            .as_deref()
            .unwrap_or(&current.provider)
            .trim()
            .to_ascii_lowercase();
        let valid_provider = match provider.as_str() {
            "cloudflare" | "netlify" | "zip" | "folder" => provider,
            _ => "zip".to_string(),
        };
        self.set_option("daypress_publish_provider", &valid_provider)?;

        if let Some(ref url) = req.production_url {
            self.set_option(
                "daypress_publish_production_url",
                url.trim().trim_end_matches('/'),
            )?;
        }
        if let Some(ref v) = req.cf_account_id {
            self.set_option("daypress_cf_account_id", v.trim())?;
        }
        if let Some(ref v) = req.cf_api_token {
            let trimmed = v.trim();
            if trimmed != MASKED_SECRET {
                self.set_option("daypress_cf_api_token", trimmed)?;
            }
        }
        if let Some(ref v) = req.cf_project_name {
            self.set_option("daypress_cf_project_name", v.trim())?;
        }
        if let Some(ref v) = req.cf_custom_domain {
            self.set_option(
                "daypress_cf_custom_domain",
                v.trim()
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_end_matches('/'),
            )?;
        }
        if let Some(ref v) = req.netlify_token {
            let trimmed = v.trim();
            if trimmed != MASKED_SECRET {
                self.set_option("daypress_netlify_token", trimmed)?;
            }
        }
        if let Some(ref v) = req.netlify_site_id {
            self.set_option("daypress_netlify_site_id", v.trim())?;
        }
        if let Some(ref v) = req.netlify_custom_domain {
            self.set_option(
                "daypress_netlify_custom_domain",
                v.trim()
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_end_matches('/'),
            )?;
        }
        if let Some(ref v) = req.local_export_dir {
            let clean = if v.trim().is_empty() {
                "data/static-export"
            } else {
                v.trim()
            };
            self.set_option("daypress_publish_export_dir", clean)?;
        }
        if let Some(b) = req.static_search_enabled {
            self.set_option("daypress_static_search_enabled", if b { "1" } else { "0" })?;
        }
        if let Some(b) = req.static_redirects_enabled {
            self.set_option(
                "daypress_static_redirects_enabled",
                if b { "1" } else { "0" },
            )?;
        }
        if let Some(ref v) = req.static_form_mode {
            self.set_option("daypress_static_form_mode", v.trim())?;
        }
        if let Some(ref v) = req.static_form_webhook_url {
            self.set_option("daypress_static_form_webhook_url", v.trim())?;
        }
        if let Some(b) = req.neutralize_admin_links {
            self.set_option("daypress_neutralize_admin_links", if b { "1" } else { "0" })?;
        }

        Ok(self.get_publish_settings())
    }

    /// Lists deployments from `daypress_deployments` ordered newest first.
    pub fn list_deployments(&self, limit: usize) -> Vec<DeploymentRecord> {
        let Ok(conn) = self.conn.lock() else {
            return vec![];
        };
        let Ok(mut stmt) = conn.prepare(
            r#"SELECT id, provider, target_url, deployment_external_id, status, commit_message,
                      pages_count, assets_count, total_bytes, bundle_path, error_message,
                      created_at, duration_ms
               FROM daypress_deployments
               ORDER BY id DESC
               LIMIT ?1;"#,
        ) else {
            return vec![];
        };

        stmt.query_map(params![limit as i64], |r| {
            Ok(DeploymentRecord {
                id: r.get(0)?,
                provider: r.get(1)?,
                target_url: r.get(2)?,
                deployment_external_id: r.get(3)?,
                status: r.get(4)?,
                commit_message: r.get(5)?,
                pages_count: r.get(6)?,
                assets_count: r.get(7)?,
                total_bytes: r.get(8)?,
                bundle_path: r.get(9)?,
                error_message: r.get(10)?,
                created_at: r.get(11)?,
                duration_ms: r.get(12)?,
            })
        })
        .ok()
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Fetches a single deployment record and its raw `manifest_json` by ID.
    pub fn get_deployment_with_manifest(
        &self,
        id: i64,
    ) -> Option<(DeploymentRecord, DeploymentManifest)> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            r#"SELECT id, provider, target_url, deployment_external_id, status, commit_message,
                      pages_count, assets_count, total_bytes, bundle_path, error_message,
                      created_at, duration_ms, manifest_json
               FROM daypress_deployments
               WHERE id = ?1;"#,
            params![id],
            |r| {
                let rec = DeploymentRecord {
                    id: r.get(0)?,
                    provider: r.get(1)?,
                    target_url: r.get(2)?,
                    deployment_external_id: r.get(3)?,
                    status: r.get(4)?,
                    commit_message: r.get(5)?,
                    pages_count: r.get(6)?,
                    assets_count: r.get(7)?,
                    total_bytes: r.get(8)?,
                    bundle_path: r.get(9)?,
                    error_message: r.get(10)?,
                    created_at: r.get(11)?,
                    duration_ms: r.get(12)?,
                };
                let manifest_raw: String = r.get(13)?;
                let manifest: DeploymentManifest =
                    serde_json::from_str(&manifest_raw).unwrap_or_default();
                Ok((rec, manifest))
            },
        )
        .ok()
    }

    /// Returns the most recent `'live'` deployment and its manifest, if any.
    pub fn get_last_live_deployment(&self) -> Option<(DeploymentRecord, DeploymentManifest)> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            r#"SELECT id, provider, target_url, deployment_external_id, status, commit_message,
                      pages_count, assets_count, total_bytes, bundle_path, error_message,
                      created_at, duration_ms, manifest_json
               FROM daypress_deployments
               WHERE status = 'live'
               ORDER BY id DESC
               LIMIT 1;"#,
            [],
            |r| {
                let rec = DeploymentRecord {
                    id: r.get(0)?,
                    provider: r.get(1)?,
                    target_url: r.get(2)?,
                    deployment_external_id: r.get(3)?,
                    status: r.get(4)?,
                    commit_message: r.get(5)?,
                    pages_count: r.get(6)?,
                    assets_count: r.get(7)?,
                    total_bytes: r.get(8)?,
                    bundle_path: r.get(9)?,
                    error_message: r.get(10)?,
                    created_at: r.get(11)?,
                    duration_ms: r.get(12)?,
                };
                let manifest_raw: String = r.get(13)?;
                let manifest: DeploymentManifest =
                    serde_json::from_str(&manifest_raw).unwrap_or_default();
                Ok((rec, manifest))
            },
        )
        .ok()
    }

    /// Inserts a completed deployment into `daypress_deployments`.
    pub fn record_deployment(
        &self,
        provider: &str,
        target_url: &str,
        external_id: Option<&str>,
        status: &str,
        commit_message: &str,
        pages_count: usize,
        assets_count: usize,
        total_bytes: usize,
        manifest: &DeploymentManifest,
        bundle_path: Option<&str>,
        error_message: Option<&str>,
        duration_ms: i64,
    ) -> Result<DeploymentRecord, AppError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let manifest_json = serde_json::to_string(manifest).unwrap_or_else(|_| "{}".to_string());
        let id = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| AppError::Internal("Database lock poisoned".to_string()))?;
            conn.execute(
                r#"INSERT INTO daypress_deployments
                   (provider, target_url, deployment_external_id, status, commit_message,
                    pages_count, assets_count, total_bytes, manifest_json, bundle_path,
                    error_message, created_at, duration_ms)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13);"#,
                params![
                    provider,
                    target_url,
                    external_id,
                    status,
                    commit_message,
                    pages_count as i64,
                    assets_count as i64,
                    total_bytes as i64,
                    manifest_json,
                    bundle_path,
                    error_message,
                    now,
                    duration_ms
                ],
            )?;
            conn.last_insert_rowid()
        };

        Ok(DeploymentRecord {
            id,
            provider: provider.to_string(),
            target_url: target_url.to_string(),
            deployment_external_id: external_id.map(|s| s.to_string()),
            status: status.to_string(),
            commit_message: commit_message.to_string(),
            pages_count: pages_count as i64,
            assets_count: assets_count as i64,
            total_bytes: total_bytes as i64,
            bundle_path: bundle_path.map(|s| s.to_string()),
            error_message: error_message.map(|s| s.to_string()),
            created_at: now,
            duration_ms,
        })
    }

    /// Computes the full `PublishStatusResponse` (unpublished changes diff, preflight warnings,
    /// provider settings, and deployment history).
    pub fn compute_publish_status(&self) -> PublishStatusResponse {
        let settings = self.get_publish_settings();
        let last_live = self.get_last_live_deployment();
        let baseline = last_live.as_ref().map(|(_, m)| m);

        let mut unpublished_changes = Vec::new();
        let mut preflight_warnings = Vec::new();

        if let Ok(conn) = self.conn.lock() {
            // 1. Check published posts/pages/CPTs against last live manifest
            if let Ok(mut stmt) = conn.prepare(
                r#"SELECT ID, post_type, post_title, post_name, post_modified, post_content, COALESCE(layout_json, '')
                   FROM posts
                   WHERE post_status = 'publish'
                     AND post_type NOT IN ('revision', 'nav_menu_item', 'attachment')
                   ORDER BY post_modified DESC, ID DESC;"#,
            ) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                    ))
                }) {
                    for (id, ptype, title, slug, modified, content, layout_json) in rows.flatten() {
                        let key = id.to_string();
                        let slug_path = if slug.is_empty() {
                            format!("/?p={id}")
                        } else {
                            format!("/{slug}/")
                        };
                        match baseline.and_then(|b| b.post_timestamps.get(&key)) {
                            None => {
                                unpublished_changes.push(UnpublishedChangeItem {
                                    kind: ptype.clone(),
                                    title: if title.is_empty() {
                                        format!("Untitled #{id}")
                                    } else {
                                        title.clone()
                                    },
                                    slug_or_path: slug_path.clone(),
                                    modified_at: modified,
                                    change_type: "added".to_string(),
                                });
                            }
                            Some(prev_mod) if prev_mod != &modified => {
                                unpublished_changes.push(UnpublishedChangeItem {
                                    kind: ptype.clone(),
                                    title: if title.is_empty() {
                                        format!("Untitled #{id}")
                                    } else {
                                        title.clone()
                                    },
                                    slug_or_path: slug_path.clone(),
                                    modified_at: modified,
                                    change_type: "modified".to_string(),
                                });
                            }
                            _ => {}
                        }

                        // Pre-flight check: detect starter `/d-admin/` CTA links
                        if content.contains("/d-admin/") || layout_json.contains("/d-admin/") {
                            preflight_warnings.push(PreflightWarning {
                                severity: "info".to_string(),
                                code: "starter_admin_link".to_string(),
                                message: format!(
                                    "{} \"{}\" ({}) contains a local `/d-admin/` link (such as the starter CTA button). {}",
                                    if ptype == "page" { "Page" } else { "Post" },
                                    title,
                                    slug_path,
                                    if settings.neutralize_admin_links {
                                        "It will be safely neutralized in the static export."
                                    } else {
                                        "Update the button URL in DayStudio before publishing."
                                    }
                                ),
                                source: slug_path,
                            });
                        }
                    }
                }
            }

            // 2. Check Theme Builder templates (`daypress_templates`)
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, title, template_type, updated_at FROM daypress_templates WHERE status = 'publish' ORDER BY id;",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                }) {
                    for (id, title, ttype, updated_at) in rows.flatten() {
                        let key = id.to_string();
                        match baseline.and_then(|b| b.template_timestamps.get(&key)) {
                            None => {
                                unpublished_changes.push(UnpublishedChangeItem {
                                    kind: "template".to_string(),
                                    title,
                                    slug_or_path: format!("Template ({ttype})"),
                                    modified_at: updated_at,
                                    change_type: "added".to_string(),
                                });
                            }
                            Some(prev_up) if prev_up != &updated_at => {
                                unpublished_changes.push(UnpublishedChangeItem {
                                    kind: "template".to_string(),
                                    title,
                                    slug_or_path: format!("Template ({ttype})"),
                                    modified_at: updated_at,
                                    change_type: "modified".to_string(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 3. Check media attachments
            if let Ok(mut stmt) = conn.prepare(
                "SELECT ID, post_title, guid, post_modified FROM posts WHERE post_type = 'attachment' ORDER BY ID DESC;",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                }) {
                    for (id, title, guid, modified) in rows.flatten() {
                        let already_published =
                            baseline.is_some_and(|b| b.media_ids.contains(&id));
                        if !already_published {
                            unpublished_changes.push(UnpublishedChangeItem {
                                kind: "media".to_string(),
                                title: if title.is_empty() {
                                    format!("Media #{id}")
                                } else {
                                    title
                                },
                                slug_or_path: guid,
                                modified_at: modified,
                                change_type: "added".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Provider readiness warnings
        match settings.provider.as_str() {
            "cloudflare" => {
                if settings.cf_account_id.trim().is_empty()
                    || settings.cf_api_token.trim().is_empty()
                    || settings.cf_project_name.trim().is_empty()
                {
                    preflight_warnings.push(PreflightWarning {
                        severity: "warning".to_string(),
                        code: "missing_provider_credentials".to_string(),
                        message: "Cloudflare Pages is selected, but Account ID, API Token, or Project Name is not configured yet.".to_string(),
                        source: "Cloudflare Pages".to_string(),
                    });
                }
            }
            "netlify"
                if settings.netlify_token.trim().is_empty()
                    || settings.netlify_site_id.trim().is_empty() =>
            {
                preflight_warnings.push(PreflightWarning {
                    severity: "warning".to_string(),
                    code: "missing_provider_credentials".to_string(),
                    message: "Netlify is selected, but Personal Access Token or Site ID is not configured yet.".to_string(),
                    source: "Netlify".to_string(),
                });
            }
            _ => {}
        }

        let deployments = self.list_deployments(25);
        PublishStatusResponse {
            settings: settings.redacted(),
            unpublished_changes,
            preflight_warnings,
            deployments,
            last_live_deployment: last_live.map(|(rec, _)| rec),
        }
    }

    /// Directory on disk where deployment `.zip` snapshots are stored.
    pub fn deployments_dir(&self) -> PathBuf {
        if self.db_path == Path::new(":memory:") {
            std::env::temp_dir().join("daypress-test-deployments")
        } else {
            self.db_path
                .parent()
                .unwrap_or_else(|| Path::new("data"))
                .join("deployments")
        }
    }
}

/// Renders an internal route on `app` as an anonymous visitor (`OptionalAuthUser(None)`).
async fn render_anonymous_route(app: &axum::Router, uri: &str) -> Option<(u16, Vec<u8>)> {
    use tower::ServiceExt;
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .body(Body::empty())
        .ok()?;
    let res = app.clone().oneshot(req).await.ok()?;
    let status = res.status().as_u16();
    let bytes = to_bytes(res.into_body(), 32 * 1024 * 1024).await.ok()?;
    Some((status, bytes.to_vec()))
}

/// Post-processes an exported HTML page:
/// - Rewrites local origin URLs (`http://localhost:8090`, `http://127.0.0.1:8090`, `local_base`) to `target_origin`.
/// - Neutralizes `/d-admin/` links when `neutralize_admin_links` is true.
/// - Enhances `<form>` elements for Netlify Forms or static webhook submission.
/// - Injects `<script src="/static/daypress-static-search.js" defer></script>` before `</body>`.
fn post_process_static_html(
    mut html: String,
    local_base: &str,
    settings: &PublishSettings,
) -> String {
    let target_origin = settings.resolved_target_origin();
    if !target_origin.is_empty() {
        let local_trimmed = local_base.trim().trim_end_matches('/');
        if !local_trimmed.is_empty() {
            html = html.replace(local_trimmed, &target_origin);
        }
        html = html
            .replace("http://localhost:8090", &target_origin)
            .replace("http://127.0.0.1:8090", &target_origin);
    }

    if settings.neutralize_admin_links {
        // Rewrite href="/d-admin/..." to href="#" data-daypress-admin-link-stripped="true"
        let mut out = String::with_capacity(html.len());
        let mut rest = html.as_str();
        let needle = "href=\"/d-admin/";
        while let Some(idx) = rest.find(needle) {
            out.push_str(&rest[..idx]);
            let after_href = &rest[idx + 6..]; // starts at `/d-admin/...`
            if let Some(end_quote) = after_href.find('"') {
                out.push_str("href=\"#\" data-daypress-admin-link-stripped=\"true\"");
                rest = &after_href[end_quote + 1..];
            } else {
                out.push_str(&rest[idx..idx + needle.len()]);
                rest = &rest[idx + needle.len()..];
            }
        }
        out.push_str(rest);
        html = out;
    }

    // Static-ready forms
    let use_netlify_forms = settings.provider.eq_ignore_ascii_case("netlify")
        || settings.static_form_mode.eq_ignore_ascii_case("netlify");
    if use_netlify_forms {
        html = html.replace(
            "<form class=\"daypress-form\"",
            "<form class=\"daypress-form\" data-netlify=\"true\" netlify",
        );
    }
    let webhook = settings.static_form_webhook_url.trim();
    if !webhook.is_empty() {
        let escaped_wh = crate::templates::utils::escape_html(webhook);
        html = html.replace(
            "<form class=\"daypress-form\"",
            &format!(
                "<form class=\"daypress-form\" action=\"{escaped_wh}\" data-static-webhook=\"{escaped_wh}\""
            ),
        );
    }

    if settings.static_search_enabled || !webhook.is_empty() {
        let script_tag = "<script src=\"/static/daypress-static-search.js\" defer></script>";
        if !html.contains("daypress-static-search.js") {
            if let Some(pos) = html.rfind("</body>") {
                html.insert_str(pos, script_tag);
            } else {
                html.push_str(script_tag);
            }
        }
    }

    html
}

/// Crawls and compiles the entire Daypress website into a `BuiltStaticSite` in memory.
pub async fn build_static_site(
    db: &Database,
    settings: &PublishSettings,
) -> Result<BuiltStaticSite, AppError> {
    let app = crate::create_app(db.clone());
    let local_base = db.get_base_url();

    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut manifest = DeploymentManifest::default();
    let mut search_index_items: Vec<Value> = Vec::new();
    let mut pages_count = 0usize;
    let mut assets_count = 0usize;

    // Collect routes & metadata from SQLite
    let mut routes_to_render: Vec<(String, String)> =
        vec![("/".to_string(), "index.html".to_string())];
    let mut active_redirects: Vec<(String, String, i64)> = Vec::new();

    if let Ok(conn) = db.conn.lock() {
        // 1. Published posts, pages, CPTs
        if let Ok(mut stmt) = conn.prepare(
            r#"SELECT ID, post_type, post_title, post_name, post_excerpt, post_content, post_modified
               FROM posts
               WHERE post_status = 'publish'
                 AND post_type NOT IN ('revision', 'nav_menu_item', 'attachment')
               ORDER BY ID;"#,
        ) {
            if let Ok(rows) = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            }) {
                for (id, ptype, title, slug, excerpt, content, modified) in rows.flatten() {
                    manifest.post_timestamps.insert(id.to_string(), modified);
                    let clean_slug = slug.trim().trim_matches('/');
                    if !clean_slug.is_empty() {
                        let uri = format!("/{clean_slug}/");
                        let out_path = format!("{clean_slug}/index.html");
                        if !routes_to_render.iter().any(|(_, p)| p == &out_path) {
                            routes_to_render.push((uri.clone(), out_path));
                        }
                        let plain_excerpt = if !excerpt.trim().is_empty() {
                            strip_html_tags(&excerpt)
                        } else {
                            strip_html_tags(&content)
                                .chars()
                                .take(160)
                                .collect::<String>()
                        };
                        search_index_items.push(json!({
                            "id": id,
                            "type": ptype,
                            "title": title,
                            "slug": clean_slug,
                            "url": uri,
                            "excerpt": plain_excerpt
                        }));
                    }
                }
            }
        }

        // 2. Templates timestamps
        if let Ok(mut stmt) =
            conn.prepare("SELECT id, updated_at FROM daypress_templates WHERE status = 'publish';")
        {
            if let Ok(rows) =
                stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            {
                for (id, updated_at) in rows.flatten() {
                    manifest
                        .template_timestamps
                        .insert(id.to_string(), updated_at);
                }
            }
        }

        // 3. Media attachment IDs
        if let Ok(mut stmt) =
            conn.prepare("SELECT ID FROM posts WHERE post_type = 'attachment' ORDER BY ID;")
        {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, i64>(0)) {
                manifest.media_ids = rows.flatten().collect();
            }
        }

        // 4. Category & Tag archives
        if let Ok(mut stmt) = conn.prepare(
            r#"SELECT t.slug, tt.taxonomy
               FROM terms t
               JOIN term_taxonomy tt ON t.term_id = tt.term_id
               WHERE tt.taxonomy IN ('category', 'post_tag') AND tt.count > 0;"#,
        ) {
            if let Ok(rows) =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                for (slug, tax) in rows.flatten() {
                    let clean = slug.trim().trim_matches('/');
                    if clean.is_empty() {
                        continue;
                    }
                    let prefix = if tax == "post_tag" { "tag" } else { "category" };
                    routes_to_render.push((
                        format!("/{prefix}/{clean}/"),
                        format!("{prefix}/{clean}/index.html"),
                    ));
                }
            }
        }

        // 5. Author archives
        if let Ok(mut stmt) =
            conn.prepare("SELECT user_nicename FROM users WHERE user_nicename != '';")
        {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for nicename in rows.flatten() {
                    let clean = nicename.trim().trim_matches('/');
                    if !clean.is_empty() {
                        routes_to_render.push((
                            format!("/author/{clean}/"),
                            format!("author/{clean}/index.html"),
                        ));
                    }
                }
            }
        }

        // 6. Active Redirects
        if settings.static_redirects_enabled {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT source_path, target_url, status_code FROM daypress_redirects WHERE is_active = 1 ORDER BY id;",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                }) {
                    active_redirects = rows.flatten().collect();
                }
            }
        }
    }

    // Render all HTML routes in-memory with OptionalAuthUser(None)
    for (uri, out_path) in &routes_to_render {
        if let Some((status, bytes)) = render_anonymous_route(&app, uri).await {
            if status < 500 && !bytes.is_empty() {
                let html = String::from_utf8_lossy(&bytes).into_owned();
                let processed = post_process_static_html(html, &local_base, settings);
                files.insert(out_path.clone(), processed.into_bytes());
                pages_count += 1;
            }
        }
    }

    // Render 404.html using a dedicated probe path
    if let Some((_, bytes)) = render_anonymous_route(&app, "/__daypress_static_404_probe__").await {
        if !bytes.is_empty() {
            let html = String::from_utf8_lossy(&bytes).into_owned();
            let processed = post_process_static_html(html, &local_base, settings);
            files.insert("404.html".to_string(), processed.into_bytes());
            pages_count += 1;
        }
    }

    // Render sitemap.xml, robots.txt, and feed/index.xml
    for (uri, out_path) in [
        ("/sitemap.xml", "sitemap.xml"),
        ("/robots.txt", "robots.txt"),
        ("/feed/", "feed/index.xml"),
    ] {
        if let Some((status, bytes)) = render_anonymous_route(&app, uri).await {
            if status == 200 && !bytes.is_empty() {
                let mut text = String::from_utf8_lossy(&bytes).into_owned();
                let target_origin = settings.resolved_target_origin();
                if !target_origin.is_empty() {
                    let local_trimmed = local_base.trim().trim_end_matches('/');
                    if !local_trimmed.is_empty() {
                        text = text.replace(local_trimmed, &target_origin);
                    }
                    text = text
                        .replace("http://localhost:8090", &target_origin)
                        .replace("http://127.0.0.1:8090", &target_origin);
                }
                files.insert(out_path.to_string(), text.into_bytes());
                assets_count += 1;
            }
        }
    }

    // Emit `_redirects` (Cloudflare Pages & Netlify standard format)
    if settings.static_redirects_enabled {
        let mut lines = Vec::new();
        for (src, tgt, code) in &active_redirects {
            lines.push(format!("{} {} {}", src.trim(), tgt.trim(), code));
        }
        let redirects_content = if lines.is_empty() {
            "# Daypress Static Edge Redirects\n".to_string()
        } else {
            format!("# Daypress Static Edge Redirects\n{}\n", lines.join("\n"))
        };
        files.insert("_redirects".to_string(), redirects_content.into_bytes());
        assets_count += 1;
    }

    // Emit `search-index.json` + `/static/daypress-static-search.js`
    if settings.static_search_enabled {
        let index_bytes = serde_json::to_vec_pretty(&search_index_items).unwrap_or_default();
        files.insert("search-index.json".to_string(), index_bytes);
        files.insert(
            "static/daypress-static-search.js".to_string(),
            CLIENT_STATIC_SEARCH_JS.as_bytes().to_vec(),
        );
        assets_count += 2;
    } else if !settings.static_form_webhook_url.trim().is_empty() {
        files.insert(
            "static/daypress-static-search.js".to_string(),
            CLIENT_STATIC_SEARCH_JS.as_bytes().to_vec(),
        );
        assets_count += 1;
    }

    // Bundle embedded `/static/*` assets
    for (rel_path, bytes) in crate::static_assets::all_embedded_assets() {
        let key = format!("static/{}", rel_path.trim_start_matches('/'));
        files.insert(key, bytes.to_vec());
        assets_count += 1;
    }

    // Bundle uploaded media files from `uploads/` on disk
    let uploads_root = Path::new("uploads");
    if uploads_root.is_dir() {
        collect_dir_files(
            uploads_root,
            uploads_root,
            "uploads",
            &mut files,
            &mut assets_count,
        );
    }

    // Compute SHA-256 manifest & total bytes
    let mut total_bytes = 0usize;
    for (path, bytes) in &files {
        total_bytes += bytes.len();
        let key = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{path}")
        };
        manifest.files.insert(key, sha256_hex(bytes));
    }

    Ok(BuiltStaticSite {
        files,
        pages_count,
        assets_count,
        total_bytes,
        manifest,
    })
}

fn collect_dir_files(
    root: &Path,
    current: &Path,
    prefix: &str,
    out: &mut BTreeMap<String, Vec<u8>>,
    count: &mut usize,
) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dir_files(root, &path, prefix, out, count);
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                if let Ok(bytes) = std::fs::read(&path) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    out.insert(format!("{prefix}/{rel_str}"), bytes);
                    *count += 1;
                }
            }
        }
    }
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut inside_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                out.push(' ');
            }
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Runs the end-to-end static build, saves a snapshot `.zip`, deploys to the configured provider,
/// and records the deployment in `daypress_deployments`.
pub async fn execute_publish(
    db: &Database,
    req: &PublishRunRequest,
) -> Result<DeploymentRecord, AppError> {
    let start = Instant::now();
    let mut settings = db.get_publish_settings();
    if let Some(ref p) = req.provider_override {
        if !p.trim().is_empty() {
            settings.provider = p.trim().to_ascii_lowercase();
        }
    }

    let built = build_static_site(db, &settings).await?;
    let zip_bytes = build_zip_archive(&built.files);

    // Save snapshot `.zip` to `deployments_dir()`
    let deploy_dir = db.deployments_dir();
    let _ = std::fs::create_dir_all(&deploy_dir);
    let ts_slug = Utc::now().format("%Y%m%d-%H%M%S-%3f").to_string();
    let zip_path = deploy_dir.join(format!("deploy-{ts_slug}.zip"));
    let bundle_path_str = if std::fs::write(&zip_path, &zip_bytes).is_ok() {
        Some(zip_path.to_string_lossy().to_string())
    } else {
        None
    };

    let commit_msg = if req.commit_message.trim().is_empty() {
        format!(
            "Static publish ({} pages, {} assets)",
            built.pages_count, built.assets_count
        )
    } else {
        req.commit_message.trim().to_string()
    };

    // Execute provider push
    let deploy_outcome: Result<(String, Option<String>), String> = match settings.provider.as_str()
    {
        "cloudflare" => providers::deploy_to_cloudflare(&settings, &built.files)
            .await
            .map(|(url, ext_id)| (url, Some(ext_id))),
        "netlify" => providers::deploy_to_netlify(&settings, &built.files)
            .await
            .map(|(url, ext_id)| (url, Some(ext_id))),
        _ => {
            let export_path = if db.db_path == Path::new(":memory:")
                && settings.local_export_dir == "data/static-export"
            {
                deploy_dir.join("static-export")
            } else {
                PathBuf::from(&settings.local_export_dir)
            };
            providers::export_to_local_folder(&export_path, &built.files).map(|_| {
                let target = if !settings.production_url.trim().is_empty() {
                    settings.resolved_target_origin()
                } else {
                    export_path.to_string_lossy().to_string()
                };
                (target, Some(format!("local-{ts_slug}")))
            })
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;

    match deploy_outcome {
        Ok((target_url, ext_id)) => db.record_deployment(
            &settings.provider,
            &target_url,
            ext_id.as_deref(),
            "live",
            &commit_msg,
            built.pages_count,
            built.assets_count,
            built.total_bytes,
            &built.manifest,
            bundle_path_str.as_deref(),
            None,
            duration_ms,
        ),
        Err(err_msg) => {
            let fallback_url = settings.resolved_target_origin();
            let _ = db.record_deployment(
                &settings.provider,
                &fallback_url,
                None,
                "failed",
                &commit_msg,
                built.pages_count,
                built.assets_count,
                built.total_bytes,
                &built.manifest,
                bundle_path_str.as_deref(),
                Some(&err_msg),
                duration_ms,
            );
            Err(AppError::BadRequest(err_msg))
        }
    }
}

/// Rolls back to a previous deployment snapshot by extracting its saved `.zip` archive
/// and re-deploying those exact files to the active provider.
pub async fn execute_rollback(
    db: &Database,
    deployment_id: i64,
) -> Result<DeploymentRecord, AppError> {
    let start = Instant::now();
    let (orig, orig_manifest) = db
        .get_deployment_with_manifest(deployment_id)
        .ok_or_else(|| AppError::NotFound(format!("Deployment #{deployment_id} not found")))?;

    let bundle_path = orig.bundle_path.as_deref().ok_or_else(|| {
        AppError::BadRequest(format!(
            "Deployment #{deployment_id} does not have a stored snapshot archive."
        ))
    })?;

    let zip_bytes = std::fs::read(bundle_path).map_err(|e| {
        AppError::Internal(format!(
            "Could not read snapshot archive {bundle_path}: {e}"
        ))
    })?;
    let files = read_zip_archive(&zip_bytes).map_err(AppError::Internal)?;

    let settings = db.get_publish_settings();
    let deploy_dir = db.deployments_dir();

    let deploy_outcome: Result<(String, Option<String>), String> = match settings.provider.as_str()
    {
        "cloudflare" => providers::deploy_to_cloudflare(&settings, &files)
            .await
            .map(|(url, ext_id)| (url, Some(ext_id))),
        "netlify" => providers::deploy_to_netlify(&settings, &files)
            .await
            .map(|(url, ext_id)| (url, Some(ext_id))),
        _ => {
            let export_path = if db.db_path == Path::new(":memory:")
                && settings.local_export_dir == "data/static-export"
            {
                deploy_dir.join("static-export")
            } else {
                PathBuf::from(&settings.local_export_dir)
            };
            providers::export_to_local_folder(&export_path, &files).map(|_| {
                let target = if !orig.target_url.is_empty() {
                    orig.target_url.clone()
                } else {
                    export_path.to_string_lossy().to_string()
                };
                (target, Some(format!("rollback-to-{deployment_id}")))
            })
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;
    let commit_msg = format!(
        "Rollback to Deployment #{deployment_id} ({})",
        orig.commit_message
    );

    match deploy_outcome {
        Ok((target_url, ext_id)) => db.record_deployment(
            &settings.provider,
            &target_url,
            ext_id.as_deref(),
            "live",
            &commit_msg,
            orig.pages_count as usize,
            orig.assets_count as usize,
            orig.total_bytes as usize,
            &orig_manifest,
            Some(bundle_path),
            None,
            duration_ms,
        ),
        Err(err_msg) => Err(AppError::BadRequest(err_msg)),
    }
}
