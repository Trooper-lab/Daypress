use crate::errors::AppError;
use crate::models::{MediaDetails, MediaSizeDetail};
use axum::{
    body::Body,
    extract::Path as AxPath,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use image::image_dimensions;
use image::ImageReader;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Inspects an image file, calculates standard WordPress sizes and WebP variant,
/// and builds standard WordPress `MediaDetails` metadata.
pub fn process_image_metadata(
    file_path: &Path,
    file_name: &str,
    base_url: &str,
) -> Result<MediaDetails, AppError> {
    let clean_base = base_url.trim_end_matches('/');
    let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

    let (orig_w, orig_h) = match image_dimensions(file_path) {
        Ok(dims) => dims,
        Err(_) => {
            // Not a decodable image (e.g. PDF, SVG, or document)
            return Ok(MediaDetails {
                width: 0,
                height: 0,
                file: file_name.to_string(),
                filesize: file_size,
                sizes: HashMap::new(),
            });
        }
    };

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let mut sizes = HashMap::new();

    // 1. Thumbnail (150x150)
    let (tw, th) = calculate_fit_dimensions(orig_w, orig_h, 150, 150);
    let thumb_file = format!("{}-{}x{}.{}", stem, tw, th, ext);
    sizes.insert(
        "thumbnail".to_string(),
        MediaSizeDetail {
            file: thumb_file.clone(),
            width: tw,
            height: th,
            mime_type: mime_type.to_string(),
            source_url: format!("{}/uploads/{}", clean_base, thumb_file),
        },
    );

    // 2. Medium (max 300x300)
    if orig_w > 150 || orig_h > 150 {
        let (mw, mh) = calculate_fit_dimensions(orig_w, orig_h, 300, 300);
        let med_file = format!("{}-{}x{}.{}", stem, mw, mh, ext);
        sizes.insert(
            "medium".to_string(),
            MediaSizeDetail {
                file: med_file.clone(),
                width: mw,
                height: mh,
                mime_type: mime_type.to_string(),
                source_url: format!("{}/uploads/{}", clean_base, med_file),
            },
        );
    }

    // 3. Large (max 1024x1024)
    if orig_w > 300 || orig_h > 300 {
        let (lw, lh) = calculate_fit_dimensions(orig_w, orig_h, 1024, 1024);
        let large_file = format!("{}-{}x{}.{}", stem, lw, lh, ext);
        sizes.insert(
            "large".to_string(),
            MediaSizeDetail {
                file: large_file.clone(),
                width: lw,
                height: lh,
                mime_type: mime_type.to_string(),
                source_url: format!("{}/uploads/{}", clean_base, large_file),
            },
        );
    }

    // 4. Modern WebP variant
    let webp_file = format!("{}.webp", stem);
    sizes.insert(
        "webp".to_string(),
        MediaSizeDetail {
            file: webp_file.clone(),
            width: orig_w,
            height: orig_h,
            mime_type: "image/webp".to_string(),
            source_url: format!("{}/uploads/{}", clean_base, webp_file),
        },
    );

    Ok(MediaDetails {
        width: orig_w,
        height: orig_h,
        file: file_name.to_string(),
        filesize: file_size,
        sizes,
    })
}

/// Spawns a non-blocking background tokio task to generate thumbnail crops and modern WebP variant.
pub fn spawn_background_image_processing(file_path: PathBuf) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn_blocking(move || {
            if let Err(e) = generate_all_variants_on_disk(&file_path) {
                tracing::warn!(
                    "Background image processing completed with note for {:?}: {:?}",
                    file_path,
                    e
                );
            }
        });
    }
}

/// Synchronously generates variants on disk.
pub fn generate_all_variants_on_disk(file_path: &Path) -> Result<(), AppError> {
    if !file_path.exists() {
        return Ok(());
    }

    let parent_dir = file_path.parent().unwrap_or_else(|| Path::new("uploads"));
    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image.png");
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    // Check if decodable image
    let img = match ImageReader::open(file_path) {
        Ok(reader) => match reader.decode() {
            Ok(img) => img,
            Err(_) => return Ok(()),
        },
        Err(_) => return Ok(()),
    };

    let (w, h) = (img.width(), img.height());

    // Generate WebP
    let webp_path = parent_dir.join(format!("{}.webp", stem));
    if !webp_path.exists() {
        let _ = img.save(&webp_path);
    }

    // Standard thumbnail
    let (tw, th) = calculate_fit_dimensions(w, h, 150, 150);
    let thumb_path = parent_dir.join(format!("{}-{}x{}.{}", stem, tw, th, ext));
    if !thumb_path.exists() {
        let resized = img.thumbnail(tw, th);
        let _ = resized.save(&thumb_path);
    }

    // Medium
    if w > 150 || h > 150 {
        let (mw, mh) = calculate_fit_dimensions(w, h, 300, 300);
        let med_path = parent_dir.join(format!("{}-{}x{}.{}", stem, mw, mh, ext));
        if !med_path.exists() {
            let resized = img.thumbnail(mw, mh);
            let _ = resized.save(&med_path);
        }
    }

    Ok(())
}

/// Calculates scaled width and height preserving aspect ratio within max bounds.
pub fn calculate_fit_dimensions(orig_w: u32, orig_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if orig_w == 0 || orig_h == 0 {
        return (max_w, max_h);
    }
    let ratio = (max_w as f64 / orig_w as f64).min(max_h as f64 / orig_h as f64);
    if ratio >= 1.0 {
        (orig_w, orig_h)
    } else {
        let w = ((orig_w as f64) * ratio).round() as u32;
        let h = ((orig_h as f64) * ratio).round() as u32;
        (w.max(1), h.max(1))
    }
}

/// Sanitizes an untrusted filename or path, rejecting any `Component::ParentDir` (`..`)
/// or empty filename and extracting `.file_name()` safely so path traversal is impossible.
pub fn sanitize_filename(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return None;
    }
    let normalized = trimmed.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) || normalized.split('/').any(|seg| seg == "..")
    {
        return None;
    }
    let file_name = path.file_name()?.to_str()?.trim();
    if file_name.is_empty() || file_name == "." || file_name == ".." {
        return None;
    }
    let cleaned: String = file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        return None;
    }
    Some(cleaned)
}

/// High-performance On-the-Fly Media & CDN Handler.
/// Serves static files directly from disk with immutable caching headers.
/// If a requested thumbnail or WebP variant does not yet exist on disk,
/// it parses the dimension crop pattern, derives the original source image,
/// creates the variant dynamically on the fly, persists it to disk, and streams the result.
pub async fn handle_serve_media(AxPath(path_str): AxPath<String>) -> Result<Response, StatusCode> {
    let clean_path = path_str.trim_start_matches('/').replace('\\', "/");
    if clean_path.is_empty() || clean_path.contains('\0') {
        return Err(StatusCode::BAD_REQUEST);
    }
    let rel_path = Path::new(&clean_path);
    if rel_path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) || clean_path.split('/').any(|seg| seg == "..")
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let safe_file_name = sanitize_filename(&clean_path).ok_or(StatusCode::BAD_REQUEST)?;
    let base_uploads = Path::new("uploads");
    let full_target = base_uploads.join(&clean_path);

    // 1. Direct hit: File already exists on disk (either at relative path or flat in uploads/)
    if full_target.exists() && full_target.is_file() {
        return serve_file_response(&full_target);
    }
    let flat_target = base_uploads.join(&safe_file_name);
    if flat_target.exists() && flat_target.is_file() {
        return serve_file_response(&flat_target);
    }

    // 2. On-The-Fly Generation: Parse crop pattern or webp conversion
    // Pattern A: filename-300x200.ext -> base is filename.ext
    // Pattern B: filename.webp -> base is filename.png or filename.jpg
    for candidate_rel in [clean_path.as_str(), safe_file_name.as_str()] {
        if let Some((base_file, target_w, target_h, _target_format)) =
            parse_dynamic_media_request(candidate_rel, base_uploads)
        {
            let src_path = base_uploads.join(&base_file);
            if src_path.exists() {
                if let Ok(reader) = ImageReader::open(&src_path) {
                    if let Ok(img) = reader.decode() {
                        let resized = if let (Some(w), Some(h)) = (target_w, target_h) {
                            img.thumbnail(w, h)
                        } else {
                            img
                        };

                        let out_target = base_uploads.join(candidate_rel);
                        if let Some(p) = out_target.parent() {
                            let _ = fs::create_dir_all(p);
                        }

                        // Save on the fly so subsequent hits are pure zero-latency static hits
                        let _ = resized.save(&out_target);

                        return serve_file_response(&out_target);
                    }
                }
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

fn serve_file_response(path: &Path) -> Result<Response, StatusCode> {
    let bytes = fs::read(path).map_err(|_| StatusCode::NOT_FOUND)?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        "pdf" => "application/pdf",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "ogg" | "ogv" => "video/ogg",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

/// Dissects a requested variant URL into source file name and requested dimensions/format.
pub fn parse_dynamic_media_request(
    request_rel_path: &str,
    uploads_dir: &Path,
) -> Option<(String, Option<u32>, Option<u32>, String)> {
    let path = Path::new(request_rel_path);
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path.file_name()?.to_str()?;

    let re_crop =
        regex::Regex::new(r"^(?P<stem>.+)-(?P<w>\d+)x(?P<h>\d+)\.(?P<ext>[a-zA-Z0-9]+)$").ok()?;
    if let Some(caps) = re_crop.captures(file_name) {
        let stem = caps.name("stem")?.as_str();
        let w: u32 = caps.name("w")?.as_str().parse().ok()?;
        let h: u32 = caps.name("h")?.as_str().parse().ok()?;
        let ext = caps.name("ext")?.as_str();

        // Search for potential base files: stem.ext, stem-scaled.ext, stem.png, stem.jpg, stem.jpeg
        for base_stem in [stem.to_string(), format!("{}-scaled", stem)] {
            for candidate_ext in [ext, "png", "jpg", "jpeg", "webp"] {
                let candidate_name = format!("{}.{}", base_stem, candidate_ext);
                let candidate_rel = if parent.as_os_str().is_empty() {
                    candidate_name.clone()
                } else {
                    format!("{}/{}", parent.to_string_lossy().replace('\\', "/"), candidate_name)
                };
                if uploads_dir.join(&candidate_rel).exists() {
                    return Some((candidate_rel, Some(w), Some(h), ext.to_string()));
                }
                if !parent.as_os_str().is_empty() && uploads_dir.join(&candidate_name).exists() {
                    return Some((candidate_name, Some(w), Some(h), ext.to_string()));
                }
            }
        }
    }

    // Pattern: stem.webp -> look for stem.png, stem.jpg
    if file_name.ends_with(".webp") {
        let stem = file_name.trim_end_matches(".webp");
        for candidate_ext in ["png", "jpg", "jpeg"] {
            let candidate_name = format!("{}.{}", stem, candidate_ext);
            let candidate_rel = if parent.as_os_str().is_empty() {
                candidate_name
            } else {
                format!("{}/{}", parent.to_string_lossy().replace('\\', "/"), candidate_name)
            };
            if uploads_dir.join(&candidate_rel).exists() {
                return Some((candidate_rel, None, None, "webp".to_string()));
            }
        }
    }

    // Pattern C: Base unscaled image requested (e.g. stem.jpg), but only stem-scaled.jpg or stem-1024x1024.jpg exists
    if let (Some(stem), Some(ext)) = (
        path.file_stem().and_then(|s| s.to_str()),
        path.extension().and_then(|s| s.to_str()),
    ) {
        let dir_to_scan = if parent.as_os_str().is_empty() {
            uploads_dir.to_path_buf()
        } else {
            uploads_dir.join(parent)
        };
        if let Ok(entries) = fs::read_dir(&dir_to_scan) {
            let prefix = format!("{}-", stem.to_lowercase());
            let mut best_match: Option<(u64, String)> = None;
            for entry in entries.flatten() {
                if let Some(ename) = entry.file_name().to_str() {
                    let elower = ename.to_lowercase();
                    if elower.starts_with(&prefix) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let rel = if parent.as_os_str().is_empty() {
                            ename.to_string()
                        } else {
                            format!("{}/{}", parent.to_string_lossy().replace('\\', "/"), ename)
                        };
                        if best_match.as_ref().map(|(s, _)| size > *s).unwrap_or(true) {
                            best_match = Some((size, rel));
                        }
                    }
                }
            }
            if let Some((_, best_rel)) = best_match {
                return Some((best_rel, None, None, ext.to_string()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fit_dimensions() {
        // Scaling down keeping aspect ratio
        let (w, h) = calculate_fit_dimensions(2000, 1000, 300, 300);
        assert_eq!(w, 300);
        assert_eq!(h, 150);

        let (w, h) = calculate_fit_dimensions(800, 1200, 150, 150);
        assert_eq!(w, 100);
        assert_eq!(h, 150);

        // Small image remains unchanged
        let (w, h) = calculate_fit_dimensions(100, 80, 300, 300);
        assert_eq!(w, 100);
        assert_eq!(h, 80);
    }

    #[test]
    fn test_parse_dynamic_media_request_pattern() {
        let temp_dir = std::env::temp_dir().join("neopress_media_test");
        let _ = fs::create_dir_all(&temp_dir);
        let sample_file = temp_dir.join("header-banner.png");
        fs::write(&sample_file, b"fake png data").unwrap();

        let parsed = parse_dynamic_media_request("header-banner-300x150.png", &temp_dir);
        assert!(parsed.is_some());
        let (base, w, h, ext) = parsed.unwrap();
        assert_eq!(base, "header-banner.png");
        assert_eq!(w, Some(300));
        assert_eq!(h, Some(150));
        assert_eq!(ext, "png");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sanitize_filename_rejects_traversal() {
        assert_eq!(sanitize_filename(""), None);
        assert_eq!(sanitize_filename("   "), None);
        assert_eq!(sanitize_filename(".."), None);
        assert_eq!(sanitize_filename("../secret.txt"), None);
        assert_eq!(sanitize_filename("..\\secret.txt"), None);
        assert_eq!(sanitize_filename("2026/../../etc/passwd"), None);
        assert_eq!(sanitize_filename("/etc/passwd"), None);
        assert_eq!(
            sanitize_filename("hero-banner.webp"),
            Some("hero-banner.webp".to_string())
        );
        assert_eq!(
            sanitize_filename("2026/10/hero-banner.png"),
            Some("hero-banner.png".to_string())
        );
    }
}
