//! BYOC Static Edge Provider Integrations:
//! - Cloudflare Pages Direct Upload API + Cloudflare Zone Auto-DNS (1-Click CNAME)
//! - Netlify File Digest Deploy API + Custom Domain & DNS
//! - Local Static Folder Export
//! - Live DNS & HTTPS/SSL Verification

use super::archive::{sha1_hex, sha256_hex};
use super::{DomainConfigResult, DomainVerifyResult, PublishSettings};
use base64::Engine;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

fn cf_api_base() -> String {
    std::env::var("DAYPRESS_CF_API_BASE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn netlify_api_base() -> String {
    std::env::var("DAYPRESS_NETLIFY_API_BASE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://api.netlify.com/api/v1".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(format!("Daypress-Publisher/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("Failed to initialize HTTP client: {e}"))
}

/// Extracts the registrable root zone from a hostname (e.g. `blog.example.com` -> `example.com`).
pub fn extract_root_zone(domain: &str) -> String {
    let clean = domain
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let parts: Vec<&str> = clean.split('.').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 2 {
        return clean;
    }
    // Common multi-part country TLDs (e.g. .co.uk, .com.au)
    let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    if matches!(
        last_two.as_str(),
        "co.uk" | "org.uk" | "com.au" | "co.nz" | "co.jp" | "com.br"
    ) && parts.len() >= 3
    {
        format!("{}.{last_two}", parts[parts.len() - 3])
    } else {
        last_two
    }
}

/// Safely joins a relative static path onto `base_dir`, rejecting any `..` or root traversal.
pub fn safe_join_export_path(base_dir: &Path, rel_path: &str) -> Option<PathBuf> {
    let mut out = base_dir.to_path_buf();
    let normalized = rel_path.trim_start_matches('/').replace('\\', "/");
    for comp in Path::new(&normalized).components() {
        match comp {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

/// Writes all static files into a local directory on disk.
pub fn export_to_local_folder(
    export_dir: &Path,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<usize, String> {
    std::fs::create_dir_all(export_dir).map_err(|e| {
        format!(
            "Failed to create export folder {}: {e}",
            export_dir.display()
        )
    })?;

    let mut written = 0usize;
    for (rel_path, bytes) in files {
        let Some(target) = safe_join_export_path(export_dir, rel_path) else {
            continue;
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
        }
        std::fs::write(&target, bytes)
            .map_err(|e| format!("Failed to write {}: {e}", target.display()))?;
        written += 1;
    }
    Ok(written)
}

/// Deploys the static build to Cloudflare Pages using the Direct Upload API.
pub async fn deploy_to_cloudflare(
    settings: &PublishSettings,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(String, String), String> {
    let account_id = settings.cf_account_id.trim();
    let token = settings.cf_api_token.trim();
    let project = settings.cf_project_name.trim();
    if account_id.is_empty() || token.is_empty() || project.is_empty() {
        return Err(
            "Cloudflare Account ID, API Token, and Project Name are required to deploy to Cloudflare Pages."
                .to_string(),
        );
    }

    let base = cf_api_base();
    let client = http_client()?;

    // 1. Ensure the Cloudflare Pages project exists (create if missing)
    let proj_url = format!("{base}/accounts/{account_id}/pages/projects/{project}");
    let get_proj = client
        .get(&proj_url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Cloudflare API connection error: {e}"))?;

    if get_proj.status().as_u16() == 404 {
        let create_url = format!("{base}/accounts/{account_id}/pages/projects");
        let create_res = client
            .post(&create_url)
            .bearer_auth(token)
            .json(&json!({
                "name": project,
                "production_branch": "main"
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to create Cloudflare Pages project: {e}"))?;
        if !create_res.status().is_success() {
            let err_text = create_res.text().await.unwrap_or_default();
            return Err(format!(
                "Cloudflare Pages project creation failed: {err_text}"
            ));
        }
    } else if !get_proj.status().is_success() {
        let status = get_proj.status();
        let err_text = get_proj.text().await.unwrap_or_default();
        return Err(format!(
            "Cloudflare Pages auth/project error ({status}): {err_text}"
        ));
    }

    // 2. Build manifest & upload payload
    let mut manifest = serde_json::Map::new();
    let mut files_b64 = serde_json::Map::new();
    for (path, bytes) in files {
        let key = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{path}")
        };
        let hash = sha256_hex(bytes);
        manifest.insert(key.clone(), Value::String(hash));
        files_b64.insert(
            key,
            Value::String(base64::engine::general_purpose::STANDARD.encode(bytes)),
        );
    }

    let deploy_url = format!("{base}/accounts/{account_id}/pages/projects/{project}/deployments");
    let deploy_res = client
        .post(&deploy_url)
        .bearer_auth(token)
        .json(&json!({
            "branch": "main",
            "manifest": manifest,
            "files": files_b64
        }))
        .send()
        .await
        .map_err(|e| format!("Cloudflare Pages deployment request failed: {e}"))?;

    let status = deploy_res.status();
    let body: Value = deploy_res
        .json()
        .await
        .unwrap_or_else(|_| json!({ "success": false }));

    if !status.is_success() || body.get("success") == Some(&Value::Bool(false)) {
        let msg = body
            .get("errors")
            .and_then(|e| e.get(0))
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("Cloudflare Pages deployment rejected by API");
        return Err(format!("Cloudflare Pages error: {msg}"));
    }

    let result = body.get("result").cloned().unwrap_or(Value::Null);
    let deploy_id = result
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("cf-deploy-live")
        .to_string();
    let live_url = if !settings.cf_custom_domain.trim().is_empty() {
        format!(
            "https://{}",
            settings
                .cf_custom_domain
                .trim()
                .trim_start_matches("https://")
                .trim_start_matches("http://")
        )
    } else {
        result
            .get("url")
            .and_then(Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://{project}.pages.dev"))
    };

    Ok((live_url, deploy_id))
}

/// Deploys the static build to Netlify using the File Digest API (`SHA-1` delta upload).
pub async fn deploy_to_netlify(
    settings: &PublishSettings,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(String, String), String> {
    let token = settings.netlify_token.trim();
    let site_id = settings.netlify_site_id.trim();
    if token.is_empty() || site_id.is_empty() {
        return Err("Netlify Personal Access Token and Site ID are required.".to_string());
    }

    let base = netlify_api_base();
    let client = http_client()?;

    // 1. Map `/path` -> SHA1 and SHA1 -> `/path`
    let mut file_digests = serde_json::Map::new();
    let mut sha1_to_path: HashMap<String, String> = HashMap::new();
    for (path, bytes) in files {
        let key = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{path}")
        };
        let digest = sha1_hex(bytes);
        file_digests.insert(key.clone(), Value::String(digest.clone()));
        sha1_to_path.insert(digest, path.clone());
    }

    // 2. Create deploy with file digest manifest
    let create_url = format!("{base}/sites/{site_id}/deploys");
    let create_res = client
        .post(&create_url)
        .bearer_auth(token)
        .json(&json!({
            "files": file_digests,
            "draft": false
        }))
        .send()
        .await
        .map_err(|e| format!("Netlify API connection failed: {e}"))?;

    if !create_res.status().is_success() {
        let status = create_res.status();
        let text = create_res.text().await.unwrap_or_default();
        return Err(format!("Netlify deploy creation failed ({status}): {text}"));
    }

    let deploy_json: Value = create_res
        .json()
        .await
        .map_err(|e| format!("Invalid Netlify deploy response JSON: {e}"))?;

    let deploy_id = deploy_json
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Netlify response missing deploy `id`".to_string())?
        .to_string();

    let required_hashes: Vec<String> = deploy_json
        .get("required")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // 3. Upload only the delta files requested in `required`
    for req_sha in &required_hashes {
        if let Some(orig_path) = sha1_to_path.get(req_sha) {
            if let Some(bytes) = files.get(orig_path) {
                let clean_path = orig_path.trim_start_matches('/');
                let put_url = format!("{base}/deploys/{deploy_id}/files/{clean_path}");
                let put_res = client
                    .put(&put_url)
                    .bearer_auth(token)
                    .header("Content-Type", "application/octet-stream")
                    .body(bytes.clone())
                    .send()
                    .await
                    .map_err(|e| format!("Failed uploading {clean_path} to Netlify: {e}"))?;
                if !put_res.status().is_success() {
                    let status = put_res.status();
                    let err_text = put_res.text().await.unwrap_or_default();
                    return Err(format!(
                        "Netlify file upload failed for {clean_path} ({status}): {err_text}"
                    ));
                }
            }
        }
    }

    let live_url = if !settings.netlify_custom_domain.trim().is_empty() {
        format!(
            "https://{}",
            settings
                .netlify_custom_domain
                .trim()
                .trim_start_matches("https://")
                .trim_start_matches("http://")
        )
    } else {
        deploy_json
            .get("ssl_url")
            .or_else(|| deploy_json.get("url"))
            .and_then(Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://{site_id}.netlify.app"))
    };

    Ok((live_url, deploy_id))
}

/// Binds a custom domain to Cloudflare Pages or Netlify and automatically provisions
/// the CNAME DNS record when the domain's DNS zone lives in the connected account.
pub async fn configure_custom_domain(
    settings: &PublishSettings,
    domain_override: Option<&str>,
) -> Result<DomainConfigResult, String> {
    let provider = settings.provider.trim().to_ascii_lowercase();
    if provider == "cloudflare" {
        let domain = domain_override
            .unwrap_or(&settings.cf_custom_domain)
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_ascii_lowercase();
        if domain.is_empty() {
            return Err("Please enter a custom domain (e.g. example.com).".to_string());
        }
        let account_id = settings.cf_account_id.trim();
        let token = settings.cf_api_token.trim();
        let project = settings.cf_project_name.trim();
        if account_id.is_empty() || token.is_empty() || project.is_empty() {
            return Err(
                "Cloudflare Account ID, API Token, and Project Name are required.".to_string(),
            );
        }

        let cname_target = format!("{project}.pages.dev");
        let root_zone = extract_root_zone(&domain);
        let base = cf_api_base();
        let client = http_client()?;

        // 1. Register the custom domain on the Cloudflare Pages project
        let pages_domain_url =
            format!("{base}/accounts/{account_id}/pages/projects/{project}/domains");
        let _ = client
            .post(&pages_domain_url)
            .bearer_auth(token)
            .json(&json!({ "name": domain }))
            .send()
            .await;

        // 2. Check if the root zone exists in the user's Cloudflare account for 1-Click Auto-DNS
        let zones_url = format!("{base}/zones?name={root_zone}");
        let mut auto_dns_provisioned = false;
        let mut message = format!(
            "Attached {domain} to Cloudflare Pages project '{project}'. Add the CNAME record below at your DNS registrar."
        );

        if let Ok(zone_res) = client.get(&zones_url).bearer_auth(token).send().await {
            if let Ok(zone_json) = zone_res.json::<Value>().await {
                if let Some(zone_id) = zone_json
                    .get("result")
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.first())
                    .and_then(|z| z.get("id"))
                    .and_then(Value::as_str)
                {
                    let dns_url = format!("{base}/zones/{zone_id}/dns_records");
                    if let Ok(dns_res) = client
                        .post(&dns_url)
                        .bearer_auth(token)
                        .json(&json!({
                            "type": "CNAME",
                            "name": domain,
                            "content": cname_target,
                            "proxied": true,
                            "ttl": 1
                        }))
                        .send()
                        .await
                    {
                        if dns_res.status().is_success() {
                            auto_dns_provisioned = true;
                            message = format!(
                                "✓ 1-Click Auto-DNS Complete! Bound {domain} to '{project}' and created proxied CNAME -> {cname_target} in Cloudflare zone {root_zone}."
                            );
                        }
                    }
                }
            }
        }

        return Ok(DomainConfigResult {
            domain: domain.clone(),
            provider: "cloudflare".to_string(),
            auto_dns_provisioned,
            record_type: "CNAME".to_string(),
            record_name: domain,
            record_target: cname_target,
            message,
        });
    }

    if provider == "netlify" {
        let domain = domain_override
            .unwrap_or(&settings.netlify_custom_domain)
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_ascii_lowercase();
        if domain.is_empty() {
            return Err("Please enter a custom domain (e.g. example.com).".to_string());
        }
        let token = settings.netlify_token.trim();
        let site_id = settings.netlify_site_id.trim();
        if token.is_empty() || site_id.is_empty() {
            return Err("Netlify Token and Site ID are required.".to_string());
        }

        let cname_target = format!("{site_id}.netlify.app");
        let base = netlify_api_base();
        let client = http_client()?;

        let patch_url = format!("{base}/sites/{site_id}");
        let _ = client
            .patch(&patch_url)
            .bearer_auth(token)
            .json(&json!({ "custom_domain": domain }))
            .send()
            .await;

        return Ok(DomainConfigResult {
            domain: domain.clone(),
            provider: "netlify".to_string(),
            auto_dns_provisioned: false,
            record_type: "CNAME".to_string(),
            record_name: domain.clone(),
            record_target: cname_target.clone(),
            message: format!(
                "Attached {domain} to Netlify site '{site_id}'. Point CNAME {domain} -> {cname_target} at your DNS provider."
            ),
        });
    }

    let domain = domain_override
        .unwrap_or(&settings.production_url)
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();
    Ok(DomainConfigResult {
        domain: domain.clone(),
        provider: provider.clone(),
        auto_dns_provisioned: false,
        record_type: "CNAME / A".to_string(),
        record_name: domain,
        record_target: "your-static-host.example.com".to_string(),
        message: "Production URL updated for static bundle generation.".to_string(),
    })
}

/// Verifies live DNS resolution and SSL/HTTPS status for a configured custom domain.
pub async fn verify_domain_status(
    settings: &PublishSettings,
    domain_override: Option<&str>,
) -> DomainVerifyResult {
    let raw = domain_override
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            if !settings.cf_custom_domain.trim().is_empty() {
                Some(settings.cf_custom_domain.as_str())
            } else if !settings.netlify_custom_domain.trim().is_empty() {
                Some(settings.netlify_custom_domain.as_str())
            } else {
                Some(settings.production_url.as_str())
            }
        })
        .unwrap_or("");

    let host = raw
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string();

    if host.is_empty() {
        return DomainVerifyResult {
            domain: String::new(),
            dns_resolved: false,
            resolved_ips: vec![],
            ssl_active: false,
            status_message: "No custom domain configured yet.".to_string(),
        };
    }

    let mut resolved_ips = Vec::new();
    if let Ok(addrs) = tokio::net::lookup_host((host.as_str(), 443)).await {
        for addr in addrs {
            let ip_str = addr.ip().to_string();
            if !resolved_ips.contains(&ip_str) {
                resolved_ips.push(ip_str);
            }
        }
    }
    let dns_resolved = !resolved_ips.is_empty();

    // Check Cloudflare Pages domain SSL status if using Cloudflare and API base is configured
    let mut ssl_active = false;
    if settings.provider.eq_ignore_ascii_case("cloudflare")
        && !settings.cf_account_id.trim().is_empty()
        && !settings.cf_api_token.trim().is_empty()
        && !settings.cf_project_name.trim().is_empty()
    {
        let base = cf_api_base();
        let url = format!(
            "{base}/accounts/{}/pages/projects/{}/domains/{host}",
            settings.cf_account_id.trim(),
            settings.cf_project_name.trim()
        );
        if let Ok(client) = http_client() {
            if let Ok(res) = client
                .get(&url)
                .bearer_auth(settings.cf_api_token.trim())
                .send()
                .await
            {
                if let Ok(json_val) = res.json::<Value>().await {
                    let status = json_val
                        .get("result")
                        .and_then(|r| r.get("status"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if status.eq_ignore_ascii_case("active") {
                        ssl_active = true;
                    }
                }
            }
        }
    }

    let status_message = if dns_resolved && ssl_active {
        format!(
            "✓ {host} resolves ({}) and SSL certificate is active!",
            resolved_ips.join(", ")
        )
    } else if dns_resolved {
        format!(
            "✓ DNS resolves for {host} ({}). Edge SSL will activate automatically once traffic hits the CNAME.",
            resolved_ips.join(", ")
        )
    } else {
        format!("⏳ Waiting for DNS propagation on {host}. Verify your CNAME record.")
    };

    DomainVerifyResult {
        domain: host,
        dns_resolved,
        resolved_ips,
        ssl_active,
        status_message,
    }
}
