use crate::errors::AppError;
use crate::models::{MediaDetails, MediaSizeDetail};
use axum::{
    body::Body,
    extract::Path as AxPath,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use image::codecs::avif::AvifEncoder;
use image::image_dimensions;
use image::ImageReader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Inspects an image file, calculates standard WordPress sizes plus WebP and AVIF variants,
/// and builds standard WordPress `MediaDetails` metadata.
pub fn process_image_metadata(
    file_path: &Path,
    file_name: &str,
    base_url: &str,
) -> Result<MediaDetails, AppError> {
    process_image_metadata_with_focal(file_path, file_name, base_url, 0.5, 0.5)
}

/// Inspects an image file with custom focal point coordinates `(focal_x, focal_y)` in `[0.0, 1.0]`,
/// calculates standard WordPress sizes plus WebP and AVIF variants, and builds `MediaDetails`.
pub fn process_image_metadata_with_focal(
    file_path: &Path,
    file_name: &str,
    base_url: &str,
    focal_x: f32,
    focal_y: f32,
) -> Result<MediaDetails, AppError> {
    let fx = focal_x.clamp(0.0, 1.0);
    let fy = focal_y.clamp(0.0, 1.0);
    let clean_base = base_url.trim_end_matches('/');
    let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

    let (orig_w, orig_h) = match image_dimensions(file_path) {
        Ok(dims) => dims,
        Err(_) => {
            // Not a decodable raster image (e.g. PDF, SVG, or document)
            return Ok(MediaDetails {
                width: 0,
                height: 0,
                file: file_name.to_string(),
                filesize: file_size,
                sizes: HashMap::new(),
                focal_x: fx,
                focal_y: fy,
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
        "avif" => "image/avif",
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

    // 5. Next-Gen AVIF variant
    let avif_file = format!("{}.avif", stem);
    sizes.insert(
        "avif".to_string(),
        MediaSizeDetail {
            file: avif_file.clone(),
            width: orig_w,
            height: orig_h,
            mime_type: "image/avif".to_string(),
            source_url: format!("{}/uploads/{}", clean_base, avif_file),
        },
    );

    Ok(MediaDetails {
        width: orig_w,
        height: orig_h,
        file: file_name.to_string(),
        filesize: file_size,
        sizes,
        focal_x: fx,
        focal_y: fy,
    })
}

/// Spawns a non-blocking background tokio task to generate thumbnail crops and modern WebP/AVIF variants.
pub fn spawn_background_image_processing(file_path: PathBuf) {
    spawn_background_image_processing_with_focal(file_path, 0.5, 0.5);
}

/// Spawns a non-blocking background tokio task to generate focal-cropped thumbnails and modern WebP/AVIF variants.
pub fn spawn_background_image_processing_with_focal(
    file_path: PathBuf,
    focal_x: f32,
    focal_y: f32,
) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn_blocking(move || {
            if let Err(e) = generate_all_variants_with_focal(&file_path, focal_x, focal_y) {
                tracing::warn!(
                    "Background image processing completed with note for {:?}: {:?}",
                    file_path,
                    e
                );
            }
        });
    }
}

/// Synchronously generates variants on disk using the default center focal point `(0.5, 0.5)`.
pub fn generate_all_variants_on_disk(file_path: &Path) -> Result<(), AppError> {
    generate_all_variants_with_focal(file_path, 0.5, 0.5)
}

/// Synchronously generates WebP, AVIF, and thumbnail variants on disk with focal-point cropping.
pub fn generate_all_variants_with_focal(
    file_path: &Path,
    focal_x: f32,
    focal_y: f32,
) -> Result<(), AppError> {
    if !file_path.exists() {
        return Ok(());
    }

    let fx = focal_x.clamp(0.0, 1.0);
    let fy = focal_y.clamp(0.0, 1.0);
    let has_custom_focal = (fx - 0.5).abs() > 0.001 || (fy - 0.5).abs() > 0.001;

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

    // Generate AVIF (downscaled to max 512px with fastest speed=10 so debug builds remain fast)
    let avif_path = parent_dir.join(format!("{}.avif", stem));
    if !avif_path.exists() {
        let avif_source = if w > 512 || h > 512 {
            img.thumbnail(512, 512)
        } else {
            img.clone()
        };
        let _ = save_avif_fast(&avif_source, &avif_path);
    }

    // Standard thumbnail
    let (tw, th) = calculate_fit_dimensions(w, h, 150, 150);
    let thumb_path = parent_dir.join(format!("{}-{}x{}.{}", stem, tw, th, ext));
    if !thumb_path.exists() || has_custom_focal {
        let resized = if has_custom_focal {
            crop_with_focal_point(&img, tw, th, fx, fy)
        } else {
            img.thumbnail(tw, th)
        };
        let _ = resized.save(&thumb_path);
    }

    // Square 150x150 focal crop when image is larger than 150x150 and non-square
    if w >= 150 && h >= 150 && (tw != 150 || th != 150) {
        let sq_thumb_path = parent_dir.join(format!("{}-150x150.{}", stem, ext));
        if !sq_thumb_path.exists() || has_custom_focal {
            let cropped = crop_with_focal_point(&img, 150, 150, fx, fy);
            let _ = cropped.save(&sq_thumb_path);
        }
    }

    // Medium
    if w > 150 || h > 150 {
        let (mw, mh) = calculate_fit_dimensions(w, h, 300, 300);
        let med_path = parent_dir.join(format!("{}-{}x{}.{}", stem, mw, mh, ext));
        if !med_path.exists() || has_custom_focal {
            let resized = if has_custom_focal {
                crop_with_focal_point(&img, mw, mh, fx, fy)
            } else {
                img.thumbnail(mw, mh)
            };
            let _ = resized.save(&med_path);
        }
    }

    Ok(())
}

/// Fast AVIF file encoder using speed=10 so unoptimized test builds never stall.
fn save_avif_fast(img: &image::DynamicImage, target: &Path) -> Result<(), AppError> {
    let file = fs::File::create(target)?;
    let writer = std::io::BufWriter::new(file);
    let encoder = AvifEncoder::new_with_speed_quality(writer, 10, 60);
    img.write_with_encoder(encoder)
        .map_err(|e| AppError::Internal(format!("AVIF encoding error: {}", e)))
}

/// Crops and scales `img` to `(target_w, target_h)` centered around `(focal_x, focal_y)` in `[0.0, 1.0]`
/// so the subject at `(focal_x, focal_y)` stays centered in the resulting crop.
pub fn crop_with_focal_point(
    img: &image::DynamicImage,
    target_w: u32,
    target_h: u32,
    focal_x: f32,
    focal_y: f32,
) -> image::DynamicImage {
    let (src_w, src_h) = (img.width(), img.height());
    if target_w == 0 || target_h == 0 || src_w == 0 || src_h == 0 {
        return img.clone();
    }

    let fx = focal_x.clamp(0.0, 1.0) as f64;
    let fy = focal_y.clamp(0.0, 1.0) as f64;

    let src_aspect = src_w as f64 / src_h as f64;
    let target_aspect = target_w as f64 / target_h as f64;

    let (crop_w, crop_h) = if (src_aspect - target_aspect).abs() < 1e-6 {
        // If aspect ratios match but target is smaller, take a window if custom focal point is used,
        // otherwise full frame.
        if target_w < src_w && target_h < src_h {
            (target_w, target_h)
        } else {
            (src_w, src_h)
        }
    } else if src_aspect > target_aspect {
        // Source is wider than target aspect ratio
        let ch = src_h;
        let cw = ((src_h as f64 * target_aspect).round() as u32).clamp(1, src_w);
        (cw, ch)
    } else {
        // Source is taller than target aspect ratio
        let cw = src_w;
        let ch = ((src_w as f64 / target_aspect).round() as u32).clamp(1, src_h);
        (cw, ch)
    };

    let focal_px_x = fx * (src_w as f64);
    let focal_px_y = fy * (src_h as f64);

    let max_x = src_w.saturating_sub(crop_w) as f64;
    let max_y = src_h.saturating_sub(crop_h) as f64;

    let crop_x = (focal_px_x - (crop_w as f64) / 2.0)
        .round()
        .clamp(0.0, max_x) as u32;
    let crop_y = (focal_px_y - (crop_h as f64) / 2.0)
        .round()
        .clamp(0.0, max_y) as u32;

    let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
    if cropped.width() == target_w && cropped.height() == target_h {
        cropped
    } else {
        cropped.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle)
    }
}

/// Inspects raw SVG XML bytes, strips dangerous elements (`<script>`, `<foreignObject>`,
/// `<iframe>`, `<object>`, `<embed>`) and strips event-handler attributes (`on*`) or
/// attributes containing `javascript:` / `data:text/html` URIs.
/// Rejects non-SVG or malformed XML payloads.
pub fn sanitize_svg_bytes(raw: &[u8]) -> Result<Vec<u8>, AppError> {
    let raw_str = std::str::from_utf8(raw)
        .map_err(|_| AppError::BadRequest("SVG upload is not valid UTF-8".to_string()))?;
    let trimmed = raw_str.trim();
    if trimmed.is_empty() || !trimmed.to_ascii_lowercase().contains("<svg") {
        return Err(AppError::BadRequest(
            "Invalid SVG file: missing <svg> element".to_string(),
        ));
    }

    let mut reader = Reader::from_str(trimmed);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::with_capacity(trimmed.len()));
    let mut saw_svg = false;
    let mut skip_depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if skip_depth > 0 {
                    skip_depth += 1;
                    continue;
                }
                let tag_name = e.name().as_ref().to_string();
                if is_dangerous_svg_tag(&tag_name) {
                    skip_depth = 1;
                    continue;
                }
                if is_svg_root_tag(&tag_name) {
                    saw_svg = true;
                }
                let clean_elem = sanitize_svg_element(&tag_name, &e)?;
                writer
                    .write_event(Event::Start(clean_elem))
                    .map_err(|err| AppError::BadRequest(format!("SVG write error: {}", err)))?;
            }
            Ok(Event::Empty(e)) => {
                if skip_depth > 0 {
                    continue;
                }
                let tag_name = e.name().as_ref().to_string();
                if is_dangerous_svg_tag(&tag_name) {
                    continue;
                }
                if is_svg_root_tag(&tag_name) {
                    saw_svg = true;
                }
                let clean_elem = sanitize_svg_element(&tag_name, &e)?;
                writer
                    .write_event(Event::Empty(clean_elem))
                    .map_err(|err| AppError::BadRequest(format!("SVG write error: {}", err)))?;
            }
            Ok(Event::End(e)) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                    continue;
                }
                let tag_name = e.name().as_ref().to_string();
                if is_dangerous_svg_tag(&tag_name) {
                    continue;
                }
                writer
                    .write_event(Event::End(e))
                    .map_err(|err| AppError::BadRequest(format!("SVG write error: {}", err)))?;
            }
            Ok(Event::Text(e)) => {
                if skip_depth == 0 {
                    writer
                        .write_event(Event::Text(e))
                        .map_err(|err| AppError::BadRequest(format!("SVG write error: {}", err)))?;
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if skip_depth == 0 {
                    let name = e.as_ref();
                    if matches!(name, "amp" | "lt" | "gt" | "quot" | "apos")
                        || name.starts_with('#')
                    {
                        writer
                            .write_event(Event::GeneralRef(e))
                            .map_err(|err| {
                                AppError::BadRequest(format!("SVG write error: {}", err))
                            })?;
                    }
                }
            }
            Ok(Event::Decl(e)) => {
                if skip_depth == 0 {
                    writer
                        .write_event(Event::Decl(e))
                        .map_err(|err| AppError::BadRequest(format!("SVG write error: {}", err)))?;
                }
            }
            Ok(Event::CData(_))
            | Ok(Event::Comment(_))
            | Ok(Event::PI(_))
            | Ok(Event::DocType(_)) => {
                // Strip DOCTYPE (prevents XXE), processing instructions, comments, and raw CDATA blocks
            }
            Err(err) => {
                return Err(AppError::BadRequest(format!("Malformed SVG XML: {}", err)));
            }
        }
    }

    if !saw_svg || skip_depth != 0 {
        return Err(AppError::BadRequest(
            "Invalid SVG: root <svg> element not found or unclosed tags".to_string(),
        ));
    }

    Ok(writer.into_inner())
}

fn is_svg_root_tag(tag_name: &str) -> bool {
    let lower = tag_name.trim().to_ascii_lowercase();
    let local = lower.rsplit(':').next().unwrap_or(&lower);
    local == "svg"
}

fn is_dangerous_svg_tag(tag_name: &str) -> bool {
    let lower = tag_name.trim().to_ascii_lowercase();
    let local = lower.rsplit(':').next().unwrap_or(&lower);
    matches!(
        local,
        "script"
            | "foreignobject"
            | "iframe"
            | "object"
            | "embed"
            | "applet"
            | "animate"
            | "animatemotion"
            | "animatetransform"
            | "set"
            | "handler"
            | "listener"
            | "base"
            | "meta"
            | "link"
            | "feimage"
    )
}

fn is_dangerous_svg_attribute(attr_name: &str, attr_val: &str) -> bool {
    let key_lower = attr_name.trim().to_ascii_lowercase();
    let local_key = key_lower.rsplit(':').next().unwrap_or(&key_lower);

    if key_lower.starts_with("on") || local_key.starts_with("on") {
        return true;
    }

    let normalized_val: String = attr_val
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();

    normalized_val.contains("javascript:")
        || normalized_val.contains("vbscript:")
        || normalized_val.contains("data:text/html")
        || normalized_val.contains("data:image/svg")
        || normalized_val.contains("expression(")
}

fn sanitize_svg_element<'a>(
    tag_name: &str,
    elem: &BytesStart<'a>,
) -> Result<BytesStart<'static>, AppError> {
    let mut clean = BytesStart::new(tag_name.to_string());
    for attr_res in elem.attributes() {
        let attr =
            attr_res.map_err(|e| AppError::BadRequest(format!("Invalid SVG attribute: {}", e)))?;
        let key_str = attr.key.as_ref().to_string();
        let val_str = attr
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map(|v| v.into_owned())
            .unwrap_or_else(|_| attr.value.to_string());

        if is_dangerous_svg_attribute(&key_str, &val_str) {
            continue;
        }
        clean.push_attribute((key_str.as_str(), val_str.as_str()));
    }
    Ok(clean)
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
    if clean_path
        .split('/')
        .any(|seg| seg.eq_ignore_ascii_case("form-submissions"))
    {
        return Err(StatusCode::FORBIDDEN);
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

    // 2. On-The-Fly Generation: Parse crop pattern or webp/avif conversion
    // Pattern A: filename-300x200.ext -> base is filename.ext
    // Pattern B: filename.webp / filename.avif -> base is filename.png or filename.jpg
    for candidate_rel in [clean_path.as_str(), safe_file_name.as_str()] {
        if let Some((base_file, target_w, target_h, target_format)) =
            parse_dynamic_media_request(candidate_rel, base_uploads)
        {
            let src_path = base_uploads.join(&base_file);
            if src_path.exists() {
                if let Ok(reader) = ImageReader::open(&src_path) {
                    if let Ok(img) = reader.decode() {
                        let resized = if let (Some(w), Some(h)) = (target_w, target_h) {
                            // Variants are only ever downscaled from the original.
                            if w == 0 || h == 0 || w > img.width() || h > img.height() {
                                return Err(StatusCode::NOT_FOUND);
                            }
                            img.thumbnail(w, h)
                        } else {
                            img
                        };

                        let out_target = base_uploads.join(candidate_rel);
                        let persist = match (target_w, target_h) {
                            (Some(w), Some(h)) => is_cacheable_variant_size(w, h),
                            _ => true,
                        };
                        if !persist {
                            return serve_image_in_memory(&resized, &out_target);
                        }
                        if let Some(p) = out_target.parent() {
                            let _ = fs::create_dir_all(p);
                        }

                        // Save on the fly so subsequent hits are pure zero-latency static hits
                        if target_format.eq_ignore_ascii_case("avif") {
                            let _ = save_avif_fast(&resized, &out_target);
                        } else {
                            let _ = resized.save(&out_target);
                        }

                        return serve_file_response(&out_target);
                    }
                }
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

/// Widths/heights WordPress and Daypress generate (thumbnail, medium, medium_large, large,
/// 1536, 2048). Only variants on one of these edges are written to disk, so arbitrary
/// `name-WxH.ext` requests cannot fill the uploads directory.
const CACHEABLE_VARIANT_EDGES: [u32; 6] = [150, 300, 768, 1024, 1536, 2048];

fn is_cacheable_variant_size(w: u32, h: u32) -> bool {
    CACHEABLE_VARIANT_EDGES.contains(&w) || CACHEABLE_VARIANT_EDGES.contains(&h)
}

/// Encodes a generated variant and serves it without writing it to disk.
fn serve_image_in_memory(img: &image::DynamicImage, target: &Path) -> Result<Response, StatusCode> {
    let ext = target
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut bytes = std::io::Cursor::new(Vec::new());
    if ext == "avif" {
        let encoder = AvifEncoder::new_with_speed_quality(&mut bytes, 10, 60);
        img.write_with_encoder(encoder)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        let format = image::ImageFormat::from_extension(&ext).ok_or(StatusCode::NOT_FOUND)?;
        img.write_to(&mut bytes, format)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    let mut headers = HeaderMap::new();
    if let Ok(ct) = content_type_for_ext(&ext).parse() {
        headers.insert(header::CONTENT_TYPE, ct);
    }
    if let Ok(cc) = "public, max-age=86400".parse() {
        headers.insert(header::CACHE_CONTROL, cc);
    }
    if let Ok(nosniff) = "nosniff".parse() {
        headers.insert(header::X_CONTENT_TYPE_OPTIONS, nosniff);
    }
    if let Ok(csp) = "default-src 'none'; style-src 'unsafe-inline'; sandbox".parse() {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    }
    Ok((StatusCode::OK, headers, Body::from(bytes.into_inner())).into_response())
}

fn serve_file_response(path: &Path) -> Result<Response, StatusCode> {
    let bytes = fs::read(path).map_err(|_| StatusCode::NOT_FOUND)?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content_type = content_type_for_ext(&ext);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000, immutable".parse().unwrap(),
    );
    if let Ok(nosniff) = "nosniff".parse() {
        headers.insert(header::X_CONTENT_TYPE_OPTIONS, nosniff);
    }
    if let Ok(csp) = "default-src 'none'; style-src 'unsafe-inline'; sandbox".parse() {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    }

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

fn content_type_for_ext(ext: &str) -> &'static str {
    match ext {
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
    }
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

        // Search for potential base files: stem.ext, stem-scaled.ext, stem.png, stem.jpg, stem.jpeg, stem.webp, stem.avif
        for base_stem in [stem.to_string(), format!("{}-scaled", stem)] {
            for candidate_ext in [ext, "png", "jpg", "jpeg", "webp", "avif"] {
                let candidate_name = format!("{}.{}", base_stem, candidate_ext);
                let candidate_rel = if parent.as_os_str().is_empty() {
                    candidate_name.clone()
                } else {
                    format!(
                        "{}/{}",
                        parent.to_string_lossy().replace('\\', "/"),
                        candidate_name
                    )
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

    // Pattern B1: stem.webp -> look for stem.png, stem.jpg, stem.jpeg, stem.avif
    if file_name.ends_with(".webp") {
        let stem = file_name.trim_end_matches(".webp");
        for candidate_ext in ["png", "jpg", "jpeg", "avif"] {
            let candidate_name = format!("{}.{}", stem, candidate_ext);
            let candidate_rel = if parent.as_os_str().is_empty() {
                candidate_name.clone()
            } else {
                format!(
                    "{}/{}",
                    parent.to_string_lossy().replace('\\', "/"),
                    candidate_name
                )
            };
            if uploads_dir.join(&candidate_rel).exists() {
                return Some((candidate_rel, None, None, "webp".to_string()));
            }
            if !parent.as_os_str().is_empty() && uploads_dir.join(&candidate_name).exists() {
                return Some((candidate_name, None, None, "webp".to_string()));
            }
        }
    }

    // Pattern B2: stem.avif -> look for stem.png, stem.jpg, stem.jpeg, stem.webp
    if file_name.ends_with(".avif") {
        let stem = file_name.trim_end_matches(".avif");
        for candidate_ext in ["png", "jpg", "jpeg", "webp"] {
            let candidate_name = format!("{}.{}", stem, candidate_ext);
            let candidate_rel = if parent.as_os_str().is_empty() {
                candidate_name.clone()
            } else {
                format!(
                    "{}/{}",
                    parent.to_string_lossy().replace('\\', "/"),
                    candidate_name
                )
            };
            if uploads_dir.join(&candidate_rel).exists() {
                return Some((candidate_rel, None, None, "avif".to_string()));
            }
            if !parent.as_os_str().is_empty() && uploads_dir.join(&candidate_name).exists() {
                return Some((candidate_name, None, None, "avif".to_string()));
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
    use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

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
        let temp_dir = std::env::temp_dir().join("daypress_media_test_avif_webp");
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

        let parsed_webp = parse_dynamic_media_request("header-banner.webp", &temp_dir);
        assert!(parsed_webp.is_some());
        let (base_w, w_w, h_w, ext_w) = parsed_webp.unwrap();
        assert_eq!(base_w, "header-banner.png");
        assert_eq!(w_w, None);
        assert_eq!(h_w, None);
        assert_eq!(ext_w, "webp");

        let parsed_avif = parse_dynamic_media_request("header-banner.avif", &temp_dir);
        assert!(parsed_avif.is_some());
        let (base_a, w_a, h_a, ext_a) = parsed_avif.unwrap();
        assert_eq!(base_a, "header-banner.png");
        assert_eq!(w_a, None);
        assert_eq!(h_a, None);
        assert_eq!(ext_a, "avif");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sanitize_svg_bytes_strips_xss_and_keeps_valid_paths() {
        let dirty_svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" onload="alert(1)">
            <script type="text/javascript">alert('xss');</script>
            <foreignObject width="100" height="100"><iframe src="javascript:alert(2)"></iframe></foreignObject>
            <a xlink:href="javascript:alert(3)" href="https://daypress.dev">
                <path d="M10 10 H 90 V 90 H 10 Z" fill="#ff0000" onclick="evil()" onmouseover="steal()" />
            </a>
            <image src="data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==" width="10" height="10" />
        </svg>"##;

        let cleaned_bytes =
            sanitize_svg_bytes(dirty_svg).expect("should sanitize valid SVG structure");
        let cleaned = String::from_utf8(cleaned_bytes).unwrap();

        assert!(cleaned.contains("<svg"));
        assert!(cleaned.contains("<path"));
        assert!(cleaned.contains("M10 10 H 90 V 90 H 10 Z"));
        assert!(cleaned.contains("https://daypress.dev"));
        assert!(!cleaned.to_lowercase().contains("<script"));
        assert!(!cleaned.to_lowercase().contains("foreignobject"));
        assert!(!cleaned.to_lowercase().contains("<iframe"));
        assert!(!cleaned.to_lowercase().contains("onload"));
        assert!(!cleaned.to_lowercase().contains("onclick"));
        assert!(!cleaned.to_lowercase().contains("onmouseover"));
        assert!(!cleaned.to_lowercase().contains("javascript:"));
        assert!(!cleaned.to_lowercase().contains("data:text/html"));

        // Non-SVG payload must be rejected
        assert!(
            sanitize_svg_bytes(b"<html><body><script>alert(1)</script></body></html>").is_err()
        );
        assert!(sanitize_svg_bytes(b"not xml at all").is_err());
    }

    #[test]
    fn test_crop_with_focal_point_dimensions_and_clamping() {
        // Create a 200x100 landscape image where left half (x < 100) is red and right half (x >= 100) is blue
        let mut buf = RgbaImage::new(200, 100);
        for y in 0..100 {
            for x in 0..200 {
                if x < 100 {
                    buf.put_pixel(x, y, Rgba([255, 0, 0, 255]));
                } else {
                    buf.put_pixel(x, y, Rgba([0, 0, 255, 255]));
                }
            }
        }
        let img = DynamicImage::ImageRgba8(buf);

        // Crop 50x50 focused on far left (-0.5 clamped to 0.0) -> should be pure red
        let left_crop = crop_with_focal_point(&img, 50, 50, -0.5, 0.5);
        assert_eq!(left_crop.width(), 50);
        assert_eq!(left_crop.height(), 50);
        assert_eq!(left_crop.get_pixel(25, 25), Rgba([255, 0, 0, 255]));

        // Crop 50x50 focused on far right (1.5 clamped to 1.0) -> should be pure blue
        let right_crop = crop_with_focal_point(&img, 50, 50, 1.5, 0.5);
        assert_eq!(right_crop.width(), 50);
        assert_eq!(right_crop.height(), 50);
        assert_eq!(right_crop.get_pixel(25, 25), Rgba([0, 0, 255, 255]));
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
