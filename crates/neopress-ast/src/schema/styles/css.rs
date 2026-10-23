use super::properties::StyleProperties;

impl StyleProperties {
    /// Compiles style properties into an inline CSS style string for semantic SSR rendering.
    pub fn to_css_style(&self) -> String {
        let mut css = Vec::new();

        // 1. Padding
        let pad_top = self.padding.as_ref().and_then(|p| p.top.as_deref()).or(self.padding_top.as_deref());
        let pad_right = self.padding.as_ref().and_then(|p| p.right.as_deref()).or(self.padding_right.as_deref());
        let pad_bottom = self.padding.as_ref().and_then(|p| p.bottom.as_deref()).or(self.padding_bottom.as_deref());
        let pad_left = self.padding.as_ref().and_then(|p| p.left.as_deref()).or(self.padding_left.as_deref());

        match (pad_top, pad_right, pad_bottom, pad_left) {
            (Some(t), Some(r), Some(b), Some(l)) if t == r && r == b && b == l => {
                if !t.is_empty() { css.push(format!("padding: {};", t)); }
            }
            (Some(t), Some(r), Some(b), Some(l)) => {
                css.push(format!("padding: {} {} {} {};", t, r, b, l));
            }
            _ => {
                if let Some(t) = pad_top.filter(|s| !s.is_empty()) { css.push(format!("padding-top: {};", t)); }
                if let Some(r) = pad_right.filter(|s| !s.is_empty()) { css.push(format!("padding-right: {};", r)); }
                if let Some(b) = pad_bottom.filter(|s| !s.is_empty()) { css.push(format!("padding-bottom: {};", b)); }
                if let Some(l) = pad_left.filter(|s| !s.is_empty()) { css.push(format!("padding-left: {};", l)); }
            }
        }

        // 2. Margin
        let mar_top = self.margin.as_ref().and_then(|m| m.top.as_deref()).or(self.margin_top.as_deref());
        let mar_right = self.margin.as_ref().and_then(|m| m.right.as_deref()).or(self.margin_right.as_deref());
        let mar_bottom = self.margin.as_ref().and_then(|m| m.bottom.as_deref()).or(self.margin_bottom.as_deref());
        let mar_left = self.margin.as_ref().and_then(|m| m.left.as_deref()).or(self.margin_left.as_deref());

        match (mar_top, mar_right, mar_bottom, mar_left) {
            (Some(t), Some(r), Some(b), Some(l)) if t == r && r == b && b == l => {
                if !t.is_empty() { css.push(format!("margin: {};", t)); }
            }
            (Some(t), Some(r), Some(b), Some(l)) => {
                css.push(format!("margin: {} {} {} {};", t, r, b, l));
            }
            _ => {
                if let Some(t) = mar_top.filter(|s| !s.is_empty()) { css.push(format!("margin-top: {};", t)); }
                if let Some(r) = mar_right.filter(|s| !s.is_empty()) { css.push(format!("margin-right: {};", r)); }
                if let Some(b) = mar_bottom.filter(|s| !s.is_empty()) { css.push(format!("margin-bottom: {};", b)); }
                if let Some(l) = mar_left.filter(|s| !s.is_empty()) { css.push(format!("margin-left: {};", l)); }
            }
        }

        // 3. Typography
        let typo_color = self.typography.as_ref().and_then(|t| t.color.as_deref()).or(self.text_color.as_deref());
        let typo_size = self.typography.as_ref().and_then(|t| t.font_size.as_deref()).or(self.font_size.as_deref());
        let typo_weight = self.typography.as_ref().and_then(|t| t.font_weight.as_deref()).or(self.font_weight.as_deref());
        let typo_align = self.typography.as_ref().and_then(|t| t.text_align.as_deref()).or(self.text_align.as_deref());
        let typo_lh = self.typography.as_ref().and_then(|t| t.line_height.as_deref()).or(self.line_height.as_deref());

        if let Some(c) = typo_color.filter(|s| !s.is_empty()) { css.push(format!("color: {};", c)); }
        if let Some(s) = typo_size.filter(|s| !s.is_empty()) { css.push(format!("font-size: {};", s)); }
        if let Some(w) = typo_weight.filter(|s| !s.is_empty()) { css.push(format!("font-weight: {};", w)); }
        if let Some(a) = typo_align.filter(|s| !s.is_empty()) { css.push(format!("text-align: {};", a)); }
        if let Some(lh) = typo_lh.filter(|s| !s.is_empty()) { css.push(format!("line-height: {};", lh)); }
        if let Some(ref ff) = self.font_family.as_ref().filter(|s| !s.is_empty()) {
            if ff.contains(',') || ff.starts_with("var(") {
                css.push(format!("font-family: {};", ff));
            } else {
                css.push(format!("font-family: '{}', sans-serif;", ff));
            }
        }
        if let Some(ref tt) = self.text_transform.as_ref().filter(|s| !s.is_empty()) { css.push(format!("text-transform: {};", tt)); }
        if let Some(ref fs) = self.font_style.as_ref().filter(|s| !s.is_empty()) { css.push(format!("font-style: {};", fs)); }
        if let Some(ref ls) = self.letter_spacing.as_ref().filter(|s| !s.is_empty()) { css.push(format!("letter-spacing: {};", ls)); }
        if let Some(ref td) = self.text_decoration.as_ref().filter(|s| !s.is_empty()) { css.push(format!("text-decoration: {};", td)); }
        if let Some(ref ts) = self.text_shadow.as_ref().filter(|s| !s.is_empty()) { css.push(format!("text-shadow: {};", ts)); }

        // 4. Background
        let bg_color = self.background.as_ref().and_then(|b| b.color.as_deref()).or(self.background_color.as_deref());
        if let Some(c) = bg_color.filter(|s| !s.is_empty()) { css.push(format!("background-color: {};", c)); }
        if let Some(ref bg) = self.background {
            if let Some(ref img) = bg.image_url.as_ref().filter(|s| !s.is_empty()) { css.push(format!("background-image: url('{}');", img)); }
            if let Some(ref pos) = bg.position.as_ref().filter(|s| !s.is_empty()) { css.push(format!("background-position: {};", pos)); }
            if let Some(ref sz) = bg.size.as_ref().filter(|s| !s.is_empty()) { css.push(format!("background-size: {};", sz)); }
        }
        if let Some(ref grad) = self.background_gradient.as_ref().filter(|s| !s.is_empty()) { css.push(format!("background-image: {};", grad)); }

        // 5. Borders & Box Shadow
        let border_radius = self.border.as_ref().and_then(|b| b.radius.as_deref()).or(self.border_radius.as_deref());
        if let Some(rad) = border_radius.filter(|s| !s.is_empty()) { css.push(format!("border-radius: {};", rad)); }
        if let Some(ref bdr) = self.border {
            if let Some(ref w) = bdr.width.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-width: {};", w)); }
            if let Some(ref c) = bdr.color.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-color: {};", c)); }
            if let Some(ref st) = bdr.style.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-style: {};", st)); }
        }
        if let Some(ref bt) = self.border_top.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-top: {};", bt)); }
        if let Some(ref br) = self.border_right.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-right: {};", br)); }
        if let Some(ref bb) = self.border_bottom.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-bottom: {};", bb)); }
        if let Some(ref bl) = self.border_left.as_ref().filter(|s| !s.is_empty()) { css.push(format!("border-left: {};", bl)); }
        if let Some(ref bs) = self.box_shadow.as_ref().filter(|s| !s.is_empty()) { css.push(format!("box-shadow: {};", bs)); }

        // 6. Layout & Flexbox
        if let Some(ref d) = self.display.as_ref().filter(|s| !s.is_empty()) { css.push(format!("display: {};", d)); }
        if let Some(ref fd) = self.flex_direction.as_ref().filter(|s| !s.is_empty()) { css.push(format!("flex-direction: {};", fd)); }
        if let Some(ref fw) = self.flex_wrap.as_ref().filter(|s| !s.is_empty()) { css.push(format!("flex-wrap: {};", fw)); }
        if let Some(ref jc) = self.justify_content.as_ref().filter(|s| !s.is_empty()) { css.push(format!("justify-content: {};", jc)); }
        if let Some(ref ai) = self.align_items.as_ref().filter(|s| !s.is_empty()) { css.push(format!("align-items: {};", ai)); }
        if let Some(ref g) = self.gap.as_ref().filter(|s| !s.is_empty()) { css.push(format!("gap: {};", g)); }
        if let Some(ref fl) = self.flex.as_ref().filter(|s| !s.is_empty()) { css.push(format!("flex: {};", fl)); }

        // 7. CSS Grid
        if let Some(ref gtc) = self.grid_template_columns.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-template-columns: {};", gtc)); }
        if let Some(ref gtr) = self.grid_template_rows.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-template-rows: {};", gtr)); }
        if let Some(ref gg) = self.grid_gap.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-gap: {};", gg)); }
        if let Some(ref gc) = self.grid_column.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-column: {};", gc)); }
        if let Some(ref gr) = self.grid_row.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-row: {};", gr)); }
        if let Some(ref gaf) = self.grid_auto_flow.as_ref().filter(|s| !s.is_empty()) { css.push(format!("grid-auto-flow: {};", gaf)); }
        if let Some(ref ji) = self.justify_items.as_ref().filter(|s| !s.is_empty()) { css.push(format!("justify-items: {};", ji)); }

        // 8. Positioning
        if let Some(ref pos) = self.position.as_ref().filter(|s| !s.is_empty()) { css.push(format!("position: {};", pos)); }
        if let Some(ref top) = self.top.as_ref().filter(|s| !s.is_empty()) { css.push(format!("top: {};", top)); }
        if let Some(ref right) = self.right.as_ref().filter(|s| !s.is_empty()) { css.push(format!("right: {};", right)); }
        if let Some(ref bottom) = self.bottom.as_ref().filter(|s| !s.is_empty()) { css.push(format!("bottom: {};", bottom)); }
        if let Some(ref left) = self.left.as_ref().filter(|s| !s.is_empty()) { css.push(format!("left: {};", left)); }
        if let Some(ref zi) = self.z_index.as_ref().filter(|s| !s.is_empty()) { css.push(format!("z-index: {};", zi)); }
        if let Some(ref ov) = self.overflow.as_ref().filter(|s| !s.is_empty()) { css.push(format!("overflow: {};", ov)); }

        // 9. Sizing
        if let Some(ref w) = self.width.as_ref().filter(|s| !s.is_empty()) { css.push(format!("width: {};", w)); }
        if let Some(ref min_w) = self.min_width.as_ref().filter(|s| !s.is_empty()) { css.push(format!("min-width: {};", min_w)); }
        if let Some(ref max_w) = self.max_width.as_ref().filter(|s| !s.is_empty()) { css.push(format!("max-width: {};", max_w)); }
        if let Some(ref h) = self.height.as_ref().filter(|s| !s.is_empty()) { css.push(format!("height: {};", h)); }
        if let Some(ref min_h) = self.min_height.as_ref().filter(|s| !s.is_empty()) { css.push(format!("min-height: {};", min_h)); }
        if let Some(ref max_h) = self.max_height.as_ref().filter(|s| !s.is_empty()) { css.push(format!("max-height: {};", max_h)); }
        if let Some(ref ar) = self.aspect_ratio.as_ref().filter(|s| !s.is_empty()) { css.push(format!("aspect-ratio: {};", ar)); }
        if let Some(ref of) = self.object_fit.as_ref().filter(|s| !s.is_empty()) { css.push(format!("object-fit: {};", of)); }

        // 10. Transforms & Visual Effects
        if let Some(ref op) = self.opacity.as_ref().filter(|s| !s.is_empty()) { css.push(format!("opacity: {};", op)); }
        if let Some(ref tr) = self.transform.as_ref().filter(|s| !s.is_empty()) { css.push(format!("transform: {};", tr)); }
        if let Some(ref ts) = self.transition.as_ref().filter(|s| !s.is_empty()) { css.push(format!("transition: {};", ts)); }
        if let Some(ref bf) = self.backdrop_filter.as_ref().filter(|s| !s.is_empty()) { css.push(format!("backdrop-filter: {};", bf)); }

        // 11. Visibility Toggles
        if self.hide_desktop == Some(true) {
            css.push("display: none !important;".to_string());
        }

        // 12. Custom CSS
        if let Some(ref custom) = self.custom_css.as_ref().filter(|s| !s.is_empty()) {
            css.push((*custom).clone());
        }

        css.join(" ")
    }
}
