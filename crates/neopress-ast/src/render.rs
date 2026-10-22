use crate::elementor::normalize_video_embed_url;
use crate::kit::ElementorKit;
use crate::schema::{NeopressLayout, NeopressNode, NodeKind};

/// Compiles a complete NeopressLayout into pristine semantic HTML5 for public SSR.
pub fn render_layout_to_html(layout: &NeopressLayout) -> String {
    let mut output = String::new();

    if let Some(ref gs) = layout.global_styles {
        if gs.is_object() {
            if let Ok(kit) = serde_json::from_value::<ElementorKit>(gs.clone()) {
                output.push_str("<style id=\"neopress-kit-css\">\n");
                output.push_str(&kit.compile_to_css());
                output.push_str("</style>\n");
            }
        }
    }

    output.push_str(&render_node_to_html(&layout.root));
    output
}

fn escape_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn sanitize_editor_html(raw: &str) -> String {
    raw.replace(" contenteditable=\"true\"", "")
        .replace("contenteditable=\"true\"", "")
        .replace(" draggable=\"true\"", "")
        .replace(" active-element", "")
}

fn url_encode_query(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}

fn render_form_html(
    node_id: &str,
    form_id: i64,
    title: Option<&str>,
    button_text: Option<&str>,
    endpoint: Option<&str>,
    style_attr: &str,
    inner_html: &str,
) -> String {
    let action = endpoint
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("/wp-json/neopress/v1/forms/{}/submit", form_id));

    let title_html = title
        .filter(|t| !t.trim().is_empty())
        .map(|t| format!("<h3 class=\"np-form-title\">{}</h3>\n", escape_attr(t)))
        .unwrap_or_default();

    let btn_label = button_text
        .filter(|b| !b.trim().is_empty())
        .unwrap_or("Send Message");

    let fields_html = if inner_html.trim().is_empty() {
        format!(
            "<div class=\"np-form-fields\">\n<input type=\"text\" name=\"name\" class=\"np-input\" placeholder=\"Your Name\" required />\n<input type=\"email\" name=\"email\" class=\"np-input\" placeholder=\"Email Address\" required />\n<textarea name=\"message\" class=\"np-textarea\" rows=\"4\" placeholder=\"Your Message\" required></textarea>\n<button type=\"submit\" class=\"np-button np-form-submit\">{}</button>\n</div>\n<div class=\"np-form-status\" aria-live=\"polite\"></div>",
            escape_attr(btn_label)
        )
    } else {
        inner_html.to_string()
    };

    let onsubmit = "event.preventDefault();var f=this,s=f.querySelector('.np-form-status'),d=Object.fromEntries(new FormData(f).entries());fetch(f.action,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(d)}).then(function(r){var okMsg=(s&&s.getAttribute('data-success-msg'))||'Thank you! Your message has been sent.';var redir=s&&s.getAttribute('data-redirect-url');if(s)s.textContent=r.ok?okMsg:'Submission failed.';if(r.ok){f.reset();if(redir)window.location.href=redir;}});return false;";

    format!(
        "<form id=\"{}\" class=\"np-form\" data-form-id=\"{}\" method=\"POST\" action=\"{}\" onsubmit=\"{}\"{}>\n{}<input type=\"text\" name=\"_np_hp\" style=\"display:none\" tabindex=\"-1\" autocomplete=\"off\" />\n{}\n</form>",
        escape_attr(node_id),
        form_id,
        escape_attr(&action),
        onsubmit,
        style_attr,
        title_html,
        fields_html
    )
}

fn with_extra_classes(base_class: &str, extra: Option<&str>) -> String {
    match extra.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(ext) => format!("{} {}", base_class, ext),
        None => base_class.to_string(),
    }
}

/// Recursively compiles a NeopressNode and its descendants into semantic HTML5.
pub fn render_node_to_html(node: &NeopressNode) -> String {
    let css = node.styles.to_css_style().replace('"', "'");
    let extra_cls = node.styles.css_classes.as_deref();
    let style_attr = if css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", css)
    };

    let inner_html: String = node.children.iter().map(render_node_to_html).collect();

    let is_synthetic_root = matches!(
        node.id.as_str(),
        "elem_root" | "divi_root" | "gutenberg_root" | "sec_root" | "root"
    ) || node.id.starts_with("root_");

    let has_multi_columns = node.children.len() > 1
        && node.children.iter().all(|c| {
            matches!(
                c.kind,
                NodeKind::Column | NodeKind::Flex { basis: Some(_), .. }
            )
        });

    let mut effective_css = css.clone();
    if has_multi_columns && !effective_css.contains("display:") {
        effective_css = format!(
            "display: flex; flex-direction: row; gap: 28px; align-items: flex-start; {}",
            effective_css
        )
        .trim()
        .to_string();
    }
    let effective_style_attr = if effective_css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", effective_css)
    };

    match &node.kind {
        NodeKind::Root => inner_html,
        NodeKind::Section => {
            if is_synthetic_root && effective_css.is_empty() {
                return inner_html;
            }
            let cls = with_extra_classes("np-section", extra_cls);
            format!(
                "<section id=\"{}\" class=\"{}\"{}>\n{}\n</section>",
                escape_attr(&node.id),
                escape_attr(&cls),
                effective_style_attr,
                inner_html
            )
        }
        NodeKind::Container => {
            if is_synthetic_root && effective_css.is_empty() {
                return inner_html;
            }
            let is_boxed = extra_cls
                .map(|c| c.split_whitespace().any(|w| w == "np-container-boxed"))
                .unwrap_or(false);
            let body_html = if is_boxed {
                format!("<div class=\"np-container-inner\">\n{}\n</div>", inner_html)
            } else {
                inner_html
            };
            if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                let lower = href.to_ascii_lowercase();
                let is_img = lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".png")
                    || lower.ends_with(".webp")
                    || lower.ends_with(".svg");
                let base = if is_img {
                    "np-container np-clickable-container np-lightbox-trigger"
                } else {
                    "np-container np-clickable-container"
                };
                let cls = with_extra_classes(base, extra_cls);
                format!(
                    "<a id=\"{}\" href=\"{}\" class=\"{}\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                )
            } else {
                let cls = with_extra_classes("np-container", extra_cls);
                format!(
                    "<div id=\"{}\" class=\"{}\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                )
            }
        }
        NodeKind::Column => {
            let col_style = if css.contains("flex:") || css.contains("flex-basis:") {
                css.clone()
            } else if css.is_empty() {
                "flex: 1;".to_string()
            } else {
                format!("flex: 1; {}", css)
            };
            format!(
                "<div id=\"{}\" class=\"np-column\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
                col_style.trim(),
                inner_html
            )
        }
        NodeKind::Grid { columns, gap } => {
            let grid_style = format!(
                "display: grid; grid-template-columns: repeat({}, minmax(0, 1fr)); gap: {}px; {}",
                columns, gap, css
            );
            format!(
                "<div id=\"{}\" class=\"np-grid\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
                grid_style.trim(),
                inner_html
            )
        }
        NodeKind::Flex {
            direction,
            justify,
            align,
            wrap,
            basis,
        } => {
            let wrap_str = if *wrap { "wrap" } else { "nowrap" };
            let basis_str = basis
                .as_ref()
                .map(|b| format!("flex: 1 1 {}; min-width: 0; gap: 16px; ", escape_attr(b)))
                .unwrap_or_default();
            let flex_style = format!(
                "display: flex; flex-direction: {}; justify-content: {}; align-items: {}; flex-wrap: {}; {}{}",
                escape_attr(direction),
                escape_attr(justify),
                escape_attr(align),
                wrap_str,
                basis_str,
                css
            );
            format!(
                "<div id=\"{}\" class=\"np-flex\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
                flex_style.trim(),
                inner_html
            )
        }
        NodeKind::Heading { text, level } => {
            let lvl = if *level >= 1 && *level <= 6 {
                *level
            } else {
                2
            };
            let cls = with_extra_classes("np-heading", extra_cls);
            format!(
                "<h{} id=\"{}\" class=\"{}\"{}>{}</h{}>",
                lvl,
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(text),
                lvl
            )
        }
        NodeKind::Text { content_html } => {
            let cls = with_extra_classes("np-text", extra_cls);
            format!(
                "<div id=\"{}\" class=\"{}\"{}>{}</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(content_html)
            )
        }
        NodeKind::Image {
            src,
            alt,
            width,
            height,
            link,
        } => {
            let dim_attrs = match (width, height) {
                (Some(w), Some(h)) => format!(" width=\"{}\" height=\"{}\"", w, h),
                (Some(w), None) => format!(" width=\"{}\"", w),
                (None, Some(h)) => format!(" height=\"{}\"", h),
                _ => String::new(),
            };
            let cls = with_extra_classes("np-image", extra_cls);
            let img_tag = format!(
                "<img src=\"{}\" alt=\"{}\"{} class=\"{}\"{} loading=\"lazy\" />",
                escape_attr(src),
                escape_attr(alt),
                dim_attrs,
                escape_attr(&cls),
                style_attr
            );
            if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!("<a href=\"{}\" class=\"np-image-link\">{}</a>", escape_attr(href), img_tag)
            } else {
                img_tag
            }
        }
        NodeKind::Button { text, url, target } => {
            let cls = with_extra_classes("np-button", extra_cls);
            format!(
                "<a href=\"{}\" target=\"{}\" class=\"{}\"{}>{}</a>",
                escape_attr(url),
                escape_attr(target),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(text)
            )
        }
        NodeKind::Form {
            form_id,
            title,
            button_text,
            endpoint,
        } => render_form_html(
            &node.id,
            *form_id,
            title.as_deref(),
            button_text.as_deref(),
            endpoint.as_deref(),
            &style_attr,
            &inner_html,
        ),
        NodeKind::NativeForm {
            title,
            button_text,
            endpoint,
        } => render_form_html(
            &node.id,
            1,
            Some(title.as_str()),
            Some(button_text.as_str()),
            Some(endpoint.as_str()),
            &style_attr,
            &inner_html,
        ),
        NodeKind::Benchmark {
            label,
            value,
            comparison,
        } => {
            let val_html = if !value.trim().is_empty() {
                format!("<div class=\"np-benchmark-value\">{}</div>\n", value)
            } else {
                String::new()
            };
            let comp_html = if !comparison.trim().is_empty() {
                format!(
                    "<div class=\"np-benchmark-comparison\">{}</div>",
                    comparison
                )
            } else {
                String::new()
            };
            format!(
                "<div id=\"{}\" class=\"np-benchmark\"{}>\n<div class=\"np-benchmark-label\">{}</div>\n{}{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                label,
                val_html,
                comp_html
            )
        }
        NodeKind::CallToAction {
            title,
            description,
            button_text,
            button_url,
        } => {
            let title_html = if !title.trim().is_empty() {
                format!("<h2 class=\"np-cta-title\">{}</h2>\n", title)
            } else {
                String::new()
            };
            let desc_html = if !description.trim().is_empty() {
                format!("<div class=\"np-cta-desc\">{}</div>\n", description)
            } else {
                String::new()
            };
            let btn_html = if !button_text.trim().is_empty() {
                format!(
                    "<a href=\"{}\" class=\"np-button np-cta-btn\">{}</a>",
                    escape_attr(button_url),
                    button_text
                )
            } else {
                String::new()
            };
            format!(
                "<div id=\"{}\" class=\"np-cta\"{}>\n{}{}{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                title_html,
                desc_html,
                btn_html
            )
        }
        NodeKind::PriceTable {
            title,
            price,
            period,
            features,
            button_text,
            button_url,
        } => {
            let feats_html: String = features
                .iter()
                .map(|f| format!("<li class=\"np-price-feature\">{}</li>\n", f))
                .collect();
            let period_html = if !period.trim().is_empty() {
                format!(" <span class=\"np-price-period\">{}</span>", period)
            } else {
                String::new()
            };
            let btn_html = if !button_text.trim().is_empty() {
                format!(
                    "\n<a href=\"{}\" class=\"np-button np-price-btn\">{}</a>",
                    escape_attr(button_url),
                    button_text
                )
            } else {
                String::new()
            };
            let cls = with_extra_classes("np-price-table", extra_cls);
            format!(
                "<div id=\"{}\" class=\"{}\"{}>\n<div class=\"np-price-header\"><h3 class=\"np-price-title\">{}</h3></div>\n<div class=\"np-price-amount\"><span class=\"np-price-val\">{}</span>{}</div>\n<ul class=\"np-price-features\">\n{}</ul>{}\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                title,
                price,
                period_html,
                feats_html,
                btn_html
            )
        }
        NodeKind::Spacer { height } => {
            format!(
                "<div id=\"{}\" class=\"np-spacer\" style=\"height: {}px;\"></div>",
                escape_attr(&node.id),
                height
            )
        }
        NodeKind::Divider { style } => {
            format!(
                "<hr id=\"{}\" class=\"np-divider\" style=\"border-top-style: {}; {}\" />",
                escape_attr(&node.id),
                escape_attr(style),
                css
            )
        }
        NodeKind::Icon { name, size, color } => {
            let size_style = size
                .map(|s| format!("font-size: {}px; ", s))
                .unwrap_or_default();
            let color_style = color
                .as_ref()
                .filter(|c| !c.is_empty())
                .map(|c| format!("color: {}; ", escape_attr(c)))
                .unwrap_or_default();
            let combined = format!("{}{}{}", size_style, color_style, css);
            let cls = with_extra_classes("np-icon", extra_cls);
            let span_html = format!(
                "<span id=\"{}\" class=\"{}\" data-icon=\"{}\" style=\"{}\"><i class=\"{}\" aria-hidden=\"true\"></i></span>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(name),
                combined.trim(),
                escape_attr(name)
            );
            if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                format!(
                    "<a href=\"{}\" class=\"np-icon-link\">{}</a>",
                    escape_attr(href),
                    span_html
                )
            } else {
                span_html
            }
        }
        NodeKind::Counter {
            number,
            prefix,
            suffix,
            title,
        } => {
            let pfx = prefix.as_deref().unwrap_or("");
            let sfx = suffix.as_deref().unwrap_or("");
            let title_html = title
                .as_ref()
                .map(|t| format!("<div class=\"np-counter-title\">{}</div>", t))
                .unwrap_or_default();
            format!(
                "<div id=\"{}\" class=\"np-counter\"{}>\n<div class=\"np-counter-val\">{}{}{}</div>\n{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                pfx,
                number,
                sfx,
                title_html
            )
        }
        NodeKind::Accordion { items } => {
            let mut acc_html = String::new();
            for item in items {
                acc_html.push_str(&format!(
                    "<details class=\"np-accordion-item\">\n<summary>{}</summary>\n<div class=\"np-accordion-body\">{}</div>\n</details>\n",
                    item.title, item.content_html
                ));
            }
            format!(
                "<div id=\"{}\" class=\"np-accordion\"{}>\n{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                acc_html
            )
        }
        NodeKind::Video {
            url,
            aspect_ratio,
            autoplay,
            muted,
            loop_video,
        } => {
            let ratio = if aspect_ratio.trim().is_empty() {
                "16/9"
            } else {
                aspect_ratio.as_str()
            };
            let mut embed_url = normalize_video_embed_url(url);
            let mut params = Vec::new();
            if *autoplay {
                params.push("autoplay=1");
            }
            if *muted {
                params.push("mute=1");
            }
            if *loop_video {
                params.push("loop=1");
            }
            if !params.is_empty() && !embed_url.is_empty() {
                let sep = if embed_url.contains('?') { "&" } else { "?" };
                embed_url.push_str(sep);
                embed_url.push_str(&params.join("&"));
            }
            format!(
                "<div id=\"{}\" class=\"np-video-embed\" style=\"aspect-ratio: {}; {}\">\n<iframe src=\"{}\" frameborder=\"0\" allow=\"autoplay; encrypted-media; picture-in-picture\" allowfullscreen></iframe>\n</div>",
                escape_attr(&node.id),
                escape_attr(ratio),
                css,
                escape_attr(&embed_url)
            )
        }
        NodeKind::Map { address, zoom } => {
            let z = zoom.unwrap_or(14);
            let encoded = url_encode_query(address);
            format!(
                "<div id=\"{}\" class=\"np-map\" data-address=\"{}\" data-zoom=\"{}\"{}>\n<iframe src=\"https://maps.google.com/maps?q={}&amp;t=m&amp;z={}&amp;output=embed&amp;iwloc=near\" width=\"100%\" height=\"400\" style=\"border:0;\" allowfullscreen loading=\"lazy\" referrerpolicy=\"no-referrer-when-downgrade\"></iframe>\n</div>",
                escape_attr(&node.id),
                escape_attr(address),
                z,
                style_attr,
                encoded,
                z
            )
        }
        NodeKind::IconBox {
            icon,
            title,
            description,
            link,
        } => {
            let inner = format!(
                "<div class=\"np-iconbox-icon\"><i class=\"{}\"></i></div>\n<h3 class=\"np-iconbox-title\">{}</h3>\n<p class=\"np-iconbox-desc\">{}</p>",
                escape_attr(icon),
                title,
                description
            );
            if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!(
                    "<a id=\"{}\" href=\"{}\" class=\"np-iconbox\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    style_attr,
                    inner
                )
            } else {
                format!(
                    "<div id=\"{}\" class=\"np-iconbox\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    style_attr,
                    inner
                )
            }
        }
        NodeKind::ImageBox {
            image_url,
            title,
            description,
            link,
        } => {
            let inner = format!(
                "<div class=\"np-imagebox-img\"><img src=\"{}\" alt=\"{}\" loading=\"lazy\" /></div>\n<h3 class=\"np-imagebox-title\">{}</h3>\n<p class=\"np-imagebox-desc\">{}</p>",
                escape_attr(image_url),
                escape_attr(title),
                title,
                description
            );
            if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!(
                    "<a id=\"{}\" href=\"{}\" class=\"np-imagebox\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    style_attr,
                    inner
                )
            } else {
                format!(
                    "<div id=\"{}\" class=\"np-imagebox\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    style_attr,
                    inner
                )
            }
        }
        NodeKind::IconList { items } => {
            let mut list_html = String::new();
            for item in items {
                let text_part = if let Some(href) = item.link.as_ref().filter(|s| !s.is_empty()) {
                    format!("<a href=\"{}\">{}</a>", escape_attr(href), item.text)
                } else {
                    item.text.clone()
                };
                if item.icon.trim().is_empty() {
                    list_html.push_str(&format!(
                        "<li class=\"np-iconlist-item\"><span>{}</span></li>\n",
                        text_part
                    ));
                } else {
                    list_html.push_str(&format!(
                        "<li class=\"np-iconlist-item\"><i class=\"{}\"></i><span>{}</span></li>\n",
                        escape_attr(&item.icon),
                        text_part
                    ));
                }
            }
            let cls = with_extra_classes("np-iconlist", extra_cls);
            format!(
                "<ul id=\"{}\" class=\"{}\"{}>\n{}</ul>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                list_html
            )
        }
        NodeKind::ProgressBar {
            title,
            percent,
            display_text,
        } => {
            let pct_label = display_text.as_deref().unwrap_or("");
            format!(
                "<div id=\"{}\" class=\"np-progressbar\"{}>\n<div class=\"np-progress-title\"><span>{}</span><span>{}</span></div>\n<div class=\"np-progress-track\"><div class=\"np-progress-fill\" style=\"width: {}%;\"></div></div>\n</div>",
                escape_attr(&node.id),
                style_attr,
                title,
                pct_label,
                percent
            )
        }
        NodeKind::Testimonial {
            content,
            name,
            job,
            image_url,
        } => {
            let img_html = image_url
                .as_ref()
                .filter(|u| !u.is_empty())
                .map(|u| {
                    format!(
                        "<img src=\"{}\" alt=\"{}\" class=\"np-testimonial-avatar\" />",
                        escape_attr(u),
                        escape_attr(name)
                    )
                })
                .unwrap_or_default();
            format!(
                "<blockquote id=\"{}\" class=\"np-testimonial\"{}>\n<p class=\"np-testimonial-content\">\"{}\"</p>\n<div class=\"np-testimonial-meta\">\n{}\n<div><cite class=\"np-testimonial-name\">{}</cite><span class=\"np-testimonial-job\">{}</span></div>\n</div>\n</blockquote>",
                escape_attr(&node.id),
                style_attr,
                content,
                img_html,
                name,
                job
            )
        }
        NodeKind::Tabs { items } => {
            let mut nav_html = String::new();
            let mut content_html = String::new();
            let tab_onclick = "var p=this.closest('.np-tabs'),idx=Array.from(this.parentNode.children).indexOf(this);p.querySelectorAll('.np-tab-btn').forEach(function(b,i){b.classList.toggle('active',i===idx);});p.querySelectorAll('.np-tab-pane').forEach(function(pane,i){pane.classList.toggle('active',i===idx);pane.style.display=i===idx?'block':'none';});";
            for (idx, item) in items.iter().enumerate() {
                let active = if idx == 0 { " active" } else { "" };
                let display_style = if idx == 0 {
                    " style=\"display: block;\""
                } else {
                    " style=\"display: none;\""
                };
                nav_html.push_str(&format!(
                    "<button type=\"button\" class=\"np-tab-btn{}\" onclick=\"{}\">{}</button>\n",
                    active, tab_onclick, item.title
                ));
                content_html.push_str(&format!(
                    "<div class=\"np-tab-pane{}\"{}>{}</div>\n",
                    active, display_style, item.content_html
                ));
            }
            format!(
                "<div id=\"{}\" class=\"np-tabs\"{}>\n<div class=\"np-tabs-nav\">{}</div>\n<div class=\"np-tabs-content\">{}</div>\n</div>",
                escape_attr(&node.id),
                style_attr,
                nav_html,
                content_html
            )
        }
        NodeKind::Toggle { items } => {
            let mut tog_html = String::new();
            for item in items {
                tog_html.push_str(&format!(
                    "<details class=\"np-toggle-item\">\n<summary>{}</summary>\n<div class=\"np-toggle-body\">{}</div>\n</details>\n",
                    item.title, item.content_html
                ));
            }
            format!(
                "<div id=\"{}\" class=\"np-toggle\"{}>\n{}</div>",
                escape_attr(&node.id),
                style_attr,
                tog_html
            )
        }
        NodeKind::Alert {
            alert_type,
            title,
            description,
            dismissible,
        } => {
            let dismiss_btn = if *dismissible {
                "<button type=\"button\" class=\"np-alert-close\" aria-label=\"Close\" onclick=\"this.parentElement.style.display='none'\">&times;</button>"
            } else {
                ""
            };
            format!(
                "<div id=\"{}\" class=\"np-alert np-alert-{}\" role=\"alert\"{}>\n<strong>{}</strong> <span>{}</span>\n{}\n</div>",
                escape_attr(&node.id),
                escape_attr(alert_type),
                style_attr,
                title,
                description,
                dismiss_btn
            )
        }
        NodeKind::SocialIcons { items } => {
            let mut icons_html = String::new();
            for item in items {
                icons_html.push_str(&format!(
                    "<a href=\"{}\" class=\"np-social-icon\" target=\"_blank\" rel=\"noopener\"><i class=\"{}\"></i></a>\n",
                    escape_attr(&item.link),
                    escape_attr(&item.icon)
                ));
            }
            format!(
                "<div id=\"{}\" class=\"np-social-icons\"{}>\n{}</div>",
                escape_attr(&node.id),
                style_attr,
                icons_html
            )
        }
        NodeKind::Rating {
            rating,
            max_rating,
            title,
        } => {
            let mut stars = String::new();
            let rounded = rating.round() as u8;
            for i in 1..=(*max_rating) {
                if i <= rounded {
                    stars.push_str("<i class=\"fa-solid fa-star\"></i>");
                } else {
                    stars.push_str("<i class=\"fa-regular fa-star\"></i>");
                }
            }
            let title_html = title
                .as_ref()
                .map(|t| format!("<span class=\"np-rating-title\">{}</span>", t))
                .unwrap_or_default();
            format!(
                "<div id=\"{}\" class=\"np-rating\"{}>\n{}{}<span class=\"np-rating-score\">({}/{})</span>\n</div>",
                escape_attr(&node.id),
                style_attr,
                title_html,
                stars,
                rating,
                max_rating
            )
        }
        NodeKind::Audio { url } => {
            format!(
                "<div id=\"{}\" class=\"np-audio-wrapper\"{}>\n<audio controls src=\"{}\"></audio>\n</div>",
                escape_attr(&node.id),
                style_attr,
                escape_attr(url)
            )
        }
        NodeKind::MenuAnchor { anchor_id } => {
            format!(
                "<div id=\"{}\" class=\"np-menu-anchor\"></div>",
                escape_attr(anchor_id)
            )
        }
        NodeKind::CustomHtml { raw } => {
            let cleaned = sanitize_editor_html(raw);
            if !style_attr.is_empty() || extra_cls.is_some() {
                let cls = with_extra_classes("np-custom-widget", extra_cls);
                format!(
                    "<div id=\"{}\" class=\"{}\"{}>{}</div>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    style_attr,
                    cleaned
                )
            } else {
                cleaned
            }
        }
        NodeKind::ShortcodeFallback {
            tag,
            attributes: _,
            content,
        } => {
            format!(
                "<div id=\"{}\" class=\"np-shortcode-fallback\" data-shortcode=\"{}\"{}>\n{}\n</div>",
                escape_attr(&node.id),
                escape_attr(tag),
                style_attr,
                content
            )
        }
    }
}
