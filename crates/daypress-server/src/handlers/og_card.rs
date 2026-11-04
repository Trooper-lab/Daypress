//! Automatic Rust-Rendered OpenGraph Social Share Cards (`handlers/og_card.rs`).
//!
//! Dynamically generates `1200x630` OpenGraph (`og:image` / `twitter:image`) cards in
//! both raster `image/png` (for universal X/Twitter, LinkedIn, Slack, Discord, and iMessage
//! crawler compatibility) and vector `image/svg+xml`, styled strictly in the Daypress
//! Editorial Design System (Daylight Warm Paper `#F3EEE4` / Ink `#16171A` / Oxide `#A8401B`
//! or Night Desk `#16171A` / `#EFE9DE` / `#E8744A`).

use crate::adapters::database::search_fts::{estimate_reading_time_minutes, strip_html_for_fts};
use crate::db::Database;
use crate::templates::utils::escape_html;
use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use axum::{
    extract::{Path as AxPath, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use serde::Deserialize;

const FONT_NEWSREADER_SEMIBOLD: &[u8] =
    include_bytes!("../../assets/fonts/Newsreader-SemiBold.ttf");
const FONT_SCHIBSTED_MEDIUM: &[u8] =
    include_bytes!("../../assets/fonts/SchibstedGrotesk-Medium.ttf");
const FONT_IBM_PLEX_MONO_MEDIUM: &[u8] =
    include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf");

#[derive(Debug, Default, Deserialize)]
pub struct OgCardQueryParams {
    pub post_id: Option<i64>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub kicker: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<String>,
    pub theme: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OgCardSpec {
    pub site_name: String,
    pub site_domain: String,
    pub kicker: String,
    pub title: String,
    pub excerpt: String,
    pub author: String,
    pub date: String,
    pub reading_time: String,
    pub is_dark: bool,
    pub brand_hex: String,
}

impl OgCardSpec {
    pub fn from_db(
        db: &Database,
        params: &OgCardQueryParams,
        target_id_or_slug: Option<&str>,
    ) -> Self {
        let settings = db.get_settings().unwrap_or_default();
        let site_name = if settings.title.trim().is_empty() {
            "Daypress".to_string()
        } else {
            settings.title.trim().to_string()
        };
        let site_domain = settings
            .url
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string();

        let is_dark = match params.theme.as_deref().map(str::trim) {
            Some("dark") | Some("night") => true,
            Some("light") | Some("daylight") => false,
            _ => {
                let active = settings.active_theme.to_ascii_lowercase();
                active.contains("night") || active == "dark"
            }
        };
        let brand_hex = if settings.brand_color.trim().starts_with('#') {
            settings.brand_color.trim().to_string()
        } else {
            "#a8401b".to_string()
        };

        // Resolve target post if specified
        let resolved_post = if let Some(pid) = params.post_id.filter(|&id| id > 0) {
            db.get_post_by_id(pid).ok().flatten()
        } else if let Some(slug) = params.slug.as_deref().filter(|s| !s.trim().is_empty()) {
            if slug.eq_ignore_ascii_case("home") && settings.page_on_front > 0 {
                db.get_post_by_id(settings.page_on_front).ok().flatten()
            } else {
                db.get_post_by_slug(slug).ok().flatten()
            }
        } else if let Some(target) = target_id_or_slug {
            let clean = target
                .trim()
                .trim_end_matches(".png")
                .trim_end_matches(".svg");
            if clean.eq_ignore_ascii_case("home") {
                if settings.page_on_front > 0 {
                    db.get_post_by_id(settings.page_on_front).ok().flatten()
                } else {
                    None
                }
            } else if let Ok(pid) = clean.parse::<i64>() {
                db.get_post_by_id(pid).ok().flatten()
            } else {
                db.get_post_by_slug(clean).ok().flatten()
            }
        } else if settings.page_on_front > 0 {
            db.get_post_by_id(settings.page_on_front).ok().flatten()
        } else {
            None
        };

        if let Some(post) = resolved_post {
            let is_front = settings.show_on_front == "page"
                && settings.page_on_front > 0
                && post.id == settings.page_on_front;
            let raw_title = params
                .title
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if !post.seo_title.trim().is_empty() {
                        post.seo_title.trim().to_string()
                    } else if is_front && post.title.rendered.eq_ignore_ascii_case("home") {
                        if !settings.description.trim().is_empty() {
                            format!("{} — {}", site_name, settings.description.trim())
                        } else {
                            site_name.clone()
                        }
                    } else {
                        strip_html_for_fts(&post.title.rendered)
                    }
                });

            let raw_excerpt = params
                .excerpt
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if !post.seo_description.trim().is_empty() {
                        post.seo_description.trim().to_string()
                    } else if !post.excerpt.rendered.trim().is_empty() {
                        strip_html_for_fts(&post.excerpt.rendered)
                    } else {
                        let body = strip_html_for_fts(&post.content.rendered);
                        if !body.is_empty() {
                            body.chars().take(160).collect()
                        } else {
                            settings.description.clone()
                        }
                    }
                });

            let kicker = params
                .kicker
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if is_front {
                        "OFFICIAL SITE".to_string()
                    } else if let Some(first_cat) = post
                        .category_names
                        .first()
                        .filter(|c| !c.eq_ignore_ascii_case("Uncategorized"))
                    {
                        first_cat.to_uppercase()
                    } else {
                        post.post_type.to_uppercase()
                    }
                });

            let author = params
                .author
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if !post.author_name.trim().is_empty() {
                        post.author_name.clone()
                    } else {
                        site_name.clone()
                    }
                });

            let plain_body = strip_html_for_fts(&post.content.rendered);
            let (_, mins) = estimate_reading_time_minutes(&plain_body);
            let date = post.date.chars().take(10).collect::<String>();

            Self {
                site_name,
                site_domain,
                kicker,
                title: raw_title,
                excerpt: raw_excerpt,
                author,
                date,
                reading_time: format!("{} MIN READ", mins),
                is_dark,
                brand_hex,
            }
        } else {
            let title = params
                .title
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if !settings.description.trim().is_empty() {
                        format!("{} — {}", site_name, settings.description.trim())
                    } else {
                        site_name.clone()
                    }
                });
            let excerpt = params
                .excerpt
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| settings.description.clone());
            let kicker = params
                .kicker
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_uppercase())
                .unwrap_or_else(|| "DAYPRESS EDITION".to_string());
            let author = params
                .author
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .unwrap_or_else(|| site_name.clone());

            Self {
                site_name,
                site_domain,
                kicker,
                title,
                excerpt,
                author,
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                reading_time: "1 MIN READ".to_string(),
                is_dark,
                brand_hex,
            }
        }
    }
}

fn wrap_text_lines(text: &str, max_chars_per_line: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };
        if candidate_len <= max_chars_per_line || current.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            if word.chars().count() > max_chars_per_line {
                let truncated: String = word.chars().take(max_chars_per_line).collect();
                current.push_str(&truncated);
            } else {
                current.push_str(word);
            }
        } else {
            lines.push(current);
            if lines.len() >= max_lines {
                if let Some(last) = lines.last_mut() {
                    if last.chars().count() < max_chars_per_line {
                        last.push('…');
                    }
                }
                return lines;
            }
            current = word.to_string();
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    lines
}

fn parse_hex_rgba(hex: &str, fallback: [u8; 4]) -> Rgba<u8> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&h[0..2], 16),
            u8::from_str_radix(&h[2..4], 16),
            u8::from_str_radix(&h[4..6], 16),
        ) {
            return Rgba([r, g, b, 255]);
        }
    }
    Rgba(fallback)
}

/// Renders a 1200x630 SVG OpenGraph Social Card in the Daypress Editorial Design System.
pub fn render_og_card_svg(spec: &OgCardSpec) -> String {
    let (bg_paper, bg_raised, ink, ink_muted, rule_color, oxide) = if spec.is_dark {
        (
            "#111214", "#1A1C1F", "#EFE9DE", "#A7A297", "#393836", "#E8744A",
        )
    } else {
        (
            "#F3EEE4",
            "#FBF8F2",
            "#16171A",
            "#5A5852",
            "#D4CDC0",
            spec.brand_hex.as_str(),
        )
    };

    let title_lines = wrap_text_lines(&spec.title, 34, 3);
    let excerpt_lines = wrap_text_lines(&spec.excerpt, 64, 2);

    let mut title_svg = String::new();
    for (idx, line) in title_lines.iter().enumerate() {
        let y = 245 + (idx * 66);
        title_svg.push_str(&format!(
            r#"<text x="96" y="{}" fill="{}" font-family="'Newsreader', Georgia, serif" font-size="56" font-weight="600" letter-spacing="-0.02em">{}</text>"#,
            y,
            ink,
            escape_html(line)
        ));
    }

    let excerpt_base_y = 245 + (title_lines.len() * 66) + 24;
    let mut excerpt_svg = String::new();
    for (idx, line) in excerpt_lines.iter().enumerate() {
        let y = excerpt_base_y + (idx * 34);
        excerpt_svg.push_str(&format!(
            r#"<text x="96" y="{}" fill="{}" font-family="'Schibsted Grotesk', system-ui, sans-serif" font-size="23" font-weight="400">{}</text>"#,
            y,
            ink_muted,
            escape_html(line)
        ));
    }

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <rect width="1200" height="630" fill="{bg_paper}" />
  <rect x="44" y="40" width="1112" height="550" fill="{bg_raised}" stroke="{rule_color}" stroke-width="1.5" />
  <rect x="44" y="40" width="1112" height="6" fill="{ink}" />
  <rect x="44" y="46" width="220" height="4" fill="{oxide}" />
  <circle cx="96" cy="116" r="6" fill="{oxide}" />
  <text x="114" y="122" fill="{ink}" font-family="'IBM Plex Mono', monospace" font-size="16" font-weight="600" letter-spacing="0.08em">{site_name}</text>
  <rect x="96" y="148" width="{kicker_w}" height="30" rx="2" fill="{oxide}" />
  <text x="110" y="168" fill="#FFF8F0" font-family="'IBM Plex Mono', monospace" font-size="13" font-weight="600" letter-spacing="0.1em">{kicker}</text>
  {title_svg}
  {excerpt_svg}
  <line x1="96" y1="510" x2="1104" y2="510" stroke="{rule_color}" stroke-width="1" />
  <text x="96" y="548" fill="{ink}" font-family="'IBM Plex Mono', monospace" font-size="15" font-weight="500">{author}  •  {date}  •  {reading_time}</text>
  <text x="1104" y="548" text-anchor="end" fill="{oxide}" font-family="'IBM Plex Mono', monospace" font-size="15" font-weight="600">{site_domain}</text>
</svg>"##,
        bg_paper = bg_paper,
        bg_raised = bg_raised,
        ink = ink,
        rule_color = rule_color,
        oxide = oxide,
        site_name = escape_html(&spec.site_name.to_uppercase()),
        kicker_w = (spec.kicker.chars().count() * 10 + 28).max(90),
        kicker = escape_html(&spec.kicker),
        title_svg = title_svg,
        excerpt_svg = excerpt_svg,
        author = escape_html(&spec.author),
        date = escape_html(&spec.date),
        reading_time = escape_html(&spec.reading_time),
        site_domain = escape_html(&spec.site_domain),
    )
}

/// Renders a 1200x630 PNG OpenGraph Social Card in pure Rust using the Daypress Editorial palette
/// and embedded TrueType fonts (`Newsreader`, `Schibsted Grotesk`, `IBM Plex Mono`).
pub fn render_og_card_png(spec: &OgCardSpec) -> Vec<u8> {
    let width = 1200u32;
    let height = 630u32;
    let (bg_paper, bg_raised, ink, ink_muted, rule_col, oxide) = if spec.is_dark {
        (
            Rgba([17, 18, 20, 255]),
            Rgba([26, 28, 31, 255]),
            Rgba([239, 233, 222, 255]),
            Rgba([167, 162, 151, 255]),
            Rgba([57, 56, 54, 255]),
            Rgba([232, 116, 74, 255]),
        )
    } else {
        (
            Rgba([243, 238, 228, 255]),
            Rgba([251, 248, 242, 255]),
            Rgba([22, 23, 26, 255]),
            Rgba([90, 88, 82, 255]),
            Rgba([212, 205, 192, 255]),
            parse_hex_rgba(&spec.brand_hex, [168, 64, 27, 255]),
        )
    };
    let on_oxide = Rgba([255, 248, 240, 255]);

    let mut img = RgbaImage::from_pixel(width, height, bg_paper);

    let font_serif = FontRef::try_from_slice(FONT_NEWSREADER_SEMIBOLD).ok();
    let font_sans = FontRef::try_from_slice(FONT_SCHIBSTED_MEDIUM).ok();
    let font_mono = FontRef::try_from_slice(FONT_IBM_PLEX_MONO_MEDIUM).ok();

    // 1. Editorial card frame (44, 40) -> (1156, 590)
    fill_rect(&mut img, 44, 40, 1112, 550, bg_raised);
    stroke_rect(&mut img, 44, 40, 1112, 550, 2, rule_col);

    // 2. Strong top ink rule (6px) + Oxide accent bar (4px)
    fill_rect(&mut img, 44, 40, 1112, 6, ink);
    fill_rect(&mut img, 44, 46, 260, 4, oxide);

    // 3. Brand square + Site name header (IBM Plex Mono 18px)
    fill_rect(&mut img, 96, 104, 12, 12, oxide);
    if let Some(ref mono) = font_mono {
        draw_ttf_text(
            &mut img,
            mono,
            &spec.site_name.to_uppercase(),
            120.0,
            116.0,
            18.0,
            1.4,
            ink,
        );
    }

    // 4. Kicker pill badge (IBM Plex Mono 14px)
    let kicker_clean: String = spec.kicker.chars().take(32).collect();
    let kicker_text_w = font_mono
        .as_ref()
        .map(|f| measure_ttf_text(f, &kicker_clean, 14.5, 1.4))
        .unwrap_or((kicker_clean.chars().count() as f32) * 10.0);
    let kicker_w = (kicker_text_w.ceil() as u32 + 28).max(88);
    fill_rect(&mut img, 96, 142, kicker_w, 32, oxide);
    if let Some(ref mono) = font_mono {
        draw_ttf_text(
            &mut img,
            mono,
            &kicker_clean,
            110.0,
            163.0,
            14.5,
            1.4,
            on_oxide,
        );
    }

    // 5. Multi-line Headline (Newsreader SemiBold 58px)
    let title_lines = wrap_text_lines(&spec.title, 34, 3);
    if let Some(ref serif) = font_serif {
        for (idx, line) in title_lines.iter().enumerate() {
            let baseline_y = 250.0 + (idx as f32 * 64.0);
            draw_ttf_text(&mut img, serif, line, 96.0, baseline_y, 58.0, -0.6, ink);
        }
    }

    // 6. Excerpt / Subtitle (Schibsted Grotesk Medium 24px)
    let excerpt_lines = wrap_text_lines(&spec.excerpt, 64, 2);
    let excerpt_start_y = 250.0 + (title_lines.len() as f32 * 64.0) + 18.0;
    if let Some(ref sans) = font_sans {
        for (idx, line) in excerpt_lines.iter().enumerate() {
            let baseline_y = excerpt_start_y + (idx as f32 * 34.0);
            if baseline_y < 485.0 {
                draw_ttf_text(&mut img, sans, line, 96.0, baseline_y, 24.0, 0.0, ink_muted);
            }
        }
    }

    // 7. Bottom editorial divider & metadata bar (IBM Plex Mono 16px)
    fill_rect(&mut img, 96, 508, 1008, 1, rule_col);
    let meta_left = format!(
        "{}   ·   {}   ·   {}",
        spec.author.to_uppercase(),
        spec.date,
        spec.reading_time
    );
    if let Some(ref mono) = font_mono {
        draw_ttf_text(&mut img, mono, &meta_left, 96.0, 546.0, 16.0, 0.8, ink);

        let dom_clean: String = spec.site_domain.chars().take(36).collect();
        let dom_w = measure_ttf_text(mono, &dom_clean, 16.5, 0.8);
        let dom_x = (1104.0 - dom_w).max(600.0);
        draw_ttf_text(&mut img, mono, &dom_clean, dom_x, 546.0, 16.5, 0.8, oxide);
    }

    let mut png_bytes = Vec::with_capacity(64 * 1024);
    let encoder = PngEncoder::new(&mut png_bytes);
    let _ = encoder.write_image(img.as_raw(), width, height, ExtendedColorType::Rgba8);
    png_bytes
}

fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    let max_x = (x + w).min(img.width());
    let max_y = (y + h).min(img.height());
    for py in y..max_y {
        for px in x..max_x {
            img.put_pixel(px, py, color);
        }
    }
}

fn stroke_rect(
    img: &mut RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    thickness: u32,
    color: Rgba<u8>,
) {
    fill_rect(img, x, y, w, thickness, color);
    fill_rect(
        img,
        x,
        (y + h).saturating_sub(thickness),
        w,
        thickness,
        color,
    );
    fill_rect(img, x, y, thickness, h, color);
    fill_rect(
        img,
        (x + w).saturating_sub(thickness),
        y,
        thickness,
        h,
        color,
    );
}

fn measure_ttf_text(font: &FontRef<'_>, text: &str, px_size: f32, tracking: f32) -> f32 {
    let scale = PxScale::from(px_size);
    let scaled = font.as_scaled(scale);
    let mut width = 0.0f32;
    let mut prev_glyph = None;
    for ch in text.chars() {
        let gid = scaled.glyph_id(ch);
        if let Some(prev) = prev_glyph {
            width += scaled.kern(prev, gid);
        }
        width += scaled.h_advance(gid) + tracking;
        prev_glyph = Some(gid);
    }
    (width - tracking).max(0.0)
}

#[allow(clippy::too_many_arguments)]
fn draw_ttf_text(
    img: &mut RgbaImage,
    font: &FontRef<'_>,
    text: &str,
    start_x: f32,
    baseline_y: f32,
    px_size: f32,
    tracking: f32,
    color: Rgba<u8>,
) {
    let scale = PxScale::from(px_size);
    let scaled = font.as_scaled(scale);
    let mut cursor_x = start_x;
    let mut prev_glyph = None;
    let img_w = img.width() as i32;
    let img_h = img.height() as i32;

    for ch in text.chars() {
        if cursor_x >= 1116.0 {
            break;
        }
        let gid = scaled.glyph_id(ch);
        if let Some(prev) = prev_glyph {
            cursor_x += scaled.kern(prev, gid);
        }
        let glyph = gid.with_scale_and_position(scale, point(cursor_x, baseline_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                if coverage <= 0.01 {
                    return;
                }
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                if px >= 0 && px < img_w && py >= 0 && py < img_h {
                    let dst = img.get_pixel_mut(px as u32, py as u32);
                    let alpha = (coverage * (color.0[3] as f32 / 255.0)).clamp(0.0, 1.0);
                    let inv = 1.0 - alpha;
                    dst.0[0] =
                        ((color.0[0] as f32 * alpha) + (dst.0[0] as f32 * inv)).round() as u8;
                    dst.0[1] =
                        ((color.0[1] as f32 * alpha) + (dst.0[1] as f32 * inv)).round() as u8;
                    dst.0[2] =
                        ((color.0[2] as f32 * alpha) + (dst.0[2] as f32 * inv)).round() as u8;
                    dst.0[3] = 255;
                }
            });
        }
        cursor_x += scaled.h_advance(gid) + tracking;
        prev_glyph = Some(gid);
    }
}

/// `GET /og-image/{target}` — Serves dynamic 1200x630 PNG or SVG OpenGraph Social Card.
pub async fn handle_public_og_image(
    State(db): State<Database>,
    AxPath(target): AxPath<String>,
    Query(params): Query<OgCardQueryParams>,
) -> Response {
    let wants_svg = target.trim().ends_with(".svg")
        || params
            .format
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("svg"));
    let spec = OgCardSpec::from_db(&db, &params, Some(&target));
    if wants_svg {
        let svg = render_og_card_svg(&spec);
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/svg+xml; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            svg,
        )
            .into_response()
    } else {
        let png = render_og_card_png(&spec);
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            png,
        )
            .into_response()
    }
}

/// `GET /wp-json/daypress/v1/og-image` — REST API endpoint for dynamic OpenGraph Social Cards.
pub async fn handle_api_og_image(
    State(db): State<Database>,
    Query(params): Query<OgCardQueryParams>,
) -> Response {
    let wants_svg = params
        .format
        .as_deref()
        .is_some_and(|f| f.eq_ignore_ascii_case("svg"));
    let spec = OgCardSpec::from_db(&db, &params, None);
    if wants_svg {
        let svg = render_og_card_svg(&spec);
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/svg+xml; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            svg,
        )
            .into_response()
    } else {
        let png = render_og_card_png(&spec);
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            png,
        )
            .into_response()
    }
}
