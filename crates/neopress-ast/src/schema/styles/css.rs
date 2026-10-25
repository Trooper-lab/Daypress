use super::properties::StyleProperties;

/// Returns the first non-empty value of the two candidates.
///
/// Flat Visual Builder fields are passed first so that edits made in NeoStudio always win over
/// stale nested values carried over from an import (Elementor/Divi/Gutenberg nested objects).
fn pick<'a>(flat: Option<&'a str>, nested: Option<&'a str>) -> Option<&'a str> {
    flat.filter(|s| !s.trim().is_empty())
        .or_else(|| nested.filter(|s| !s.trim().is_empty()))
}

/// Wraps a raw background image value in `url('...')` unless it already is a CSS image function.
fn css_image_value(raw: &str) -> String {
    let t = raw.trim();
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("url(")
        || lower.contains("gradient(")
        || lower.starts_with("var(")
        || lower.starts_with("image-set(")
        || lower == "none"
    {
        t.to_string()
    } else {
        format!("url('{}')", t.replace('\'', "%27"))
    }
}

/// Splits a free-form declaration block (`a: b; c: d`) on top-level semicolons,
/// ignoring semicolons nested in parentheses or quotes (e.g. data URIs).
fn split_declarations(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for ch in block.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                }
                cur.push(ch);
            }
            None => match ch {
                '"' | '\'' => {
                    quote = Some(ch);
                    cur.push(ch);
                }
                '(' => {
                    depth += 1;
                    cur.push(ch);
                }
                ')' => {
                    depth -= 1;
                    cur.push(ch);
                }
                ';' if depth <= 0 => {
                    if !cur.trim().is_empty() {
                        out.push(cur.trim().to_string());
                    }
                    cur.clear();
                }
                _ => cur.push(ch),
            },
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

impl StyleProperties {
    /// Compiles style properties into an ordered list of `property: value;` declarations.
    pub fn css_declarations(&self) -> Vec<String> {
        let mut css: Vec<String> = Vec::new();
        let mut push = |prop: &str, val: Option<&str>| {
            if let Some(v) = val.filter(|s| !s.trim().is_empty()) {
                css.push(format!("{}: {};", prop, v.trim()));
            }
        };

        // 1. Padding
        let pad = self.padding.as_ref();
        let pad_top = pick(
            self.padding_top.as_deref(),
            pad.and_then(|p| p.top.as_deref()),
        );
        let pad_right = pick(
            self.padding_right.as_deref(),
            pad.and_then(|p| p.right.as_deref()),
        );
        let pad_bottom = pick(
            self.padding_bottom.as_deref(),
            pad.and_then(|p| p.bottom.as_deref()),
        );
        let pad_left = pick(
            self.padding_left.as_deref(),
            pad.and_then(|p| p.left.as_deref()),
        );
        match (pad_top, pad_right, pad_bottom, pad_left) {
            (Some(t), Some(r), Some(b), Some(l)) if t == r && r == b && b == l => {
                push("padding", Some(t))
            }
            (Some(t), Some(r), Some(b), Some(l)) => {
                let v = format!("{} {} {} {}", t, r, b, l);
                push("padding", Some(&v));
            }
            _ => {
                push("padding-top", pad_top);
                push("padding-right", pad_right);
                push("padding-bottom", pad_bottom);
                push("padding-left", pad_left);
            }
        }

        // 2. Margin
        let mar = self.margin.as_ref();
        let mar_top = pick(
            self.margin_top.as_deref(),
            mar.and_then(|m| m.top.as_deref()),
        );
        let mar_right = pick(
            self.margin_right.as_deref(),
            mar.and_then(|m| m.right.as_deref()),
        );
        let mar_bottom = pick(
            self.margin_bottom.as_deref(),
            mar.and_then(|m| m.bottom.as_deref()),
        );
        let mar_left = pick(
            self.margin_left.as_deref(),
            mar.and_then(|m| m.left.as_deref()),
        );
        match (mar_top, mar_right, mar_bottom, mar_left) {
            (Some(t), Some(r), Some(b), Some(l)) if t == r && r == b && b == l => {
                push("margin", Some(t))
            }
            (Some(t), Some(r), Some(b), Some(l)) => {
                let v = format!("{} {} {} {}", t, r, b, l);
                push("margin", Some(&v));
            }
            _ => {
                push("margin-top", mar_top);
                push("margin-right", mar_right);
                push("margin-bottom", mar_bottom);
                push("margin-left", mar_left);
            }
        }

        // 3. Typography
        let typo = self.typography.as_ref();
        push(
            "color",
            pick(
                self.text_color.as_deref(),
                typo.and_then(|t| t.color.as_deref()),
            ),
        );
        push(
            "font-size",
            pick(
                self.font_size.as_deref(),
                typo.and_then(|t| t.font_size.as_deref()),
            ),
        );
        push(
            "font-weight",
            pick(
                self.font_weight.as_deref(),
                typo.and_then(|t| t.font_weight.as_deref()),
            ),
        );
        push(
            "text-align",
            pick(
                self.text_align.as_deref(),
                typo.and_then(|t| t.text_align.as_deref()),
            ),
        );
        push(
            "line-height",
            pick(
                self.line_height.as_deref(),
                typo.and_then(|t| t.line_height.as_deref()),
            ),
        );
        if let Some(ff) = self.font_family.as_deref().filter(|s| !s.trim().is_empty()) {
            let ff = ff.trim();
            if ff.contains(',')
                || ff.starts_with("var(")
                || ff.starts_with('\'')
                || ff.starts_with('"')
            {
                push("font-family", Some(ff));
            } else {
                let v = format!("'{}', sans-serif", ff);
                push("font-family", Some(&v));
            }
        }
        push("text-transform", self.text_transform.as_deref());
        push("font-style", self.font_style.as_deref());
        push("letter-spacing", self.letter_spacing.as_deref());
        push("text-decoration", self.text_decoration.as_deref());
        push("text-shadow", self.text_shadow.as_deref());

        // 4. Background
        let bg = self.background.as_ref();
        push(
            "background-color",
            pick(
                self.background_color.as_deref(),
                bg.and_then(|b| b.color.as_deref()),
            ),
        );
        let bg_img = pick(
            self.background_image.as_deref(),
            bg.and_then(|b| b.image_url.as_deref()),
        )
        .map(css_image_value);
        let bg_grad = self
            .background_gradient
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string());
        match (bg_grad, bg_img) {
            (Some(g), Some(i)) => {
                let v = format!("{}, {}", g, i);
                push("background-image", Some(&v));
            }
            (Some(g), None) => push("background-image", Some(&g)),
            (None, Some(i)) => push("background-image", Some(&i)),
            (None, None) => {}
        }
        push(
            "background-position",
            pick(
                self.background_position.as_deref(),
                bg.and_then(|b| b.position.as_deref()),
            ),
        );
        push(
            "background-size",
            pick(
                self.background_size.as_deref(),
                bg.and_then(|b| b.size.as_deref()),
            ),
        );
        push("background-repeat", self.background_repeat.as_deref());
        push(
            "background-attachment",
            self.background_attachment.as_deref(),
        );

        // 5. Borders & Box Shadow
        let bdr = self.border.as_ref();
        push(
            "border-radius",
            pick(
                self.border_radius.as_deref(),
                bdr.and_then(|b| b.radius.as_deref()),
            ),
        );
        let b_width = pick(
            self.border_width.as_deref(),
            bdr.and_then(|b| b.width.as_deref()),
        );
        let b_style = pick(
            self.border_style.as_deref(),
            bdr.and_then(|b| b.style.as_deref()),
        );
        let b_color = pick(
            self.border_color.as_deref(),
            bdr.and_then(|b| b.color.as_deref()),
        );
        push("border-width", b_width);
        // A width without a style renders nothing in CSS (initial style is `none`); default to solid.
        let effective_style = b_style.or_else(|| {
            b_width
                .filter(|w| {
                    let n = w
                        .trim()
                        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '%');
                    n.parse::<f64>().map(|f| f > 0.0).unwrap_or(true)
                })
                .map(|_| "solid")
        });
        push("border-style", effective_style);
        push("border-color", b_color);
        push("border-top", self.border_top.as_deref());
        push("border-right", self.border_right.as_deref());
        push("border-bottom", self.border_bottom.as_deref());
        push("border-left", self.border_left.as_deref());
        push("box-shadow", self.box_shadow.as_deref());

        // 6. Layout & Flexbox
        push("display", self.display.as_deref());
        push("flex-direction", self.flex_direction.as_deref());
        push("flex-wrap", self.flex_wrap.as_deref());
        push("justify-content", self.justify_content.as_deref());
        push("align-items", self.align_items.as_deref());
        push("align-self", self.align_self.as_deref());
        push("gap", self.gap.as_deref());
        push("flex", self.flex.as_deref());

        // 7. CSS Grid
        push(
            "grid-template-columns",
            self.grid_template_columns.as_deref(),
        );
        push("grid-template-rows", self.grid_template_rows.as_deref());
        push("grid-gap", self.grid_gap.as_deref());
        push("grid-column", self.grid_column.as_deref());
        push("grid-row", self.grid_row.as_deref());
        push("grid-auto-flow", self.grid_auto_flow.as_deref());
        push("justify-items", self.justify_items.as_deref());

        // 8. Positioning
        push("position", self.position.as_deref());
        push("top", self.top.as_deref());
        push("right", self.right.as_deref());
        push("bottom", self.bottom.as_deref());
        push("left", self.left.as_deref());
        push("z-index", self.z_index.as_deref());
        push("overflow", self.overflow.as_deref());

        // 9. Sizing
        push("width", self.width.as_deref());
        push("min-width", self.min_width.as_deref());
        push("max-width", self.max_width.as_deref());
        push("height", self.height.as_deref());
        push("min-height", self.min_height.as_deref());
        push("max-height", self.max_height.as_deref());
        push("aspect-ratio", self.aspect_ratio.as_deref());
        push("object-fit", self.object_fit.as_deref());

        // 10. Transforms & Visual Effects
        push("opacity", self.opacity.as_deref());
        push("transform", self.transform.as_deref());
        push("transition", self.transition.as_deref());
        push("filter", self.filter.as_deref());
        push("backdrop-filter", self.backdrop_filter.as_deref());
        push("cursor", self.cursor.as_deref());

        // 11. Custom CSS declarations (appended last so they win)
        if let Some(custom) = self.custom_css.as_ref().filter(|s| !s.trim().is_empty()) {
            for decl in split_declarations(custom) {
                css.push(format!("{};", decl));
            }
        }

        css
    }

    /// Compiles style properties into an inline CSS style string for semantic SSR rendering.
    ///
    /// Responsive visibility (`hide_desktop` / `hide_tablet` / `hide_mobile`) is intentionally
    /// NOT emitted inline: it is compiled into breakpoint-scoped rules by `compile_layout_css`
    /// so hiding an element on desktop never hides it on tablet/mobile.
    pub fn to_css_style(&self) -> String {
        self.css_declarations().join(" ")
    }

    /// Same as [`to_css_style`](Self::to_css_style) but with every declaration flagged
    /// `!important`, so scoped state/breakpoint rules can override the base inline styles.
    pub fn to_css_style_important(&self) -> String {
        self.css_declarations()
            .into_iter()
            .map(|d| {
                let body = d.trim_end_matches(';').trim();
                if body.to_ascii_lowercase().contains("!important") {
                    format!("{};", body)
                } else {
                    format!("{} !important;", body)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// True when any scoped rule (state, breakpoint override or responsive visibility)
    /// targets this element, meaning the rendered element must carry its `id`.
    pub fn has_scoped_rules(&self) -> bool {
        self.hover.is_some()
            || self.focus.is_some()
            || self.active.is_some()
            || self.tablet.is_some()
            || self.mobile.is_some()
            || self.hide_desktop == Some(true)
            || self.hide_tablet == Some(true)
            || self.hide_mobile == Some(true)
    }

    /// Merged class list from `css_classes` and the legacy `custom_classes` key.
    pub fn merged_classes(&self) -> Option<String> {
        let mut out: Vec<&str> = Vec::new();
        for src in [self.css_classes.as_deref(), self.custom_classes.as_deref()]
            .into_iter()
            .flatten()
        {
            for c in src.split_whitespace() {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out.join(" "))
        }
    }
}
