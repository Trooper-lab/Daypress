use super::properties::StyleProperties;

impl StyleProperties {
    /// Synchronizes flat Visual Builder style fields and nested canonical AST style structs
    /// so both representations stay in complete lockstep, with recursive synchronization
    /// for tablet, mobile, and hover overrides.
    pub fn sync_flat_and_nested(&mut self) {
        // 1. Background color
        if let Some(ref bg) = self.background {
            if self.background_color.is_none() {
                self.background_color = bg.color.clone();
            }
        }
        if let Some(ref bg_col) = self.background_color {
            let bg = self.background.get_or_insert_with(Default::default);
            if bg.color.is_none() {
                bg.color = Some(bg_col.clone());
            }
        }

        // 2. Typography
        if let Some(ref typo) = self.typography {
            if self.text_color.is_none() {
                self.text_color = typo.color.clone();
            }
            if self.font_size.is_none() {
                self.font_size = typo.font_size.clone();
            }
            if self.font_weight.is_none() {
                self.font_weight = typo.font_weight.clone();
            }
            if self.text_align.is_none() {
                self.text_align = typo.text_align.clone();
            }
            if self.line_height.is_none() {
                self.line_height = typo.line_height.clone();
            }
        }
        if self.text_color.is_some()
            || self.font_size.is_some()
            || self.font_weight.is_some()
            || self.text_align.is_some()
            || self.line_height.is_some()
        {
            let typo = self.typography.get_or_insert_with(Default::default);
            if typo.color.is_none() {
                typo.color = self.text_color.clone();
            }
            if typo.font_size.is_none() {
                typo.font_size = self.font_size.clone();
            }
            if typo.font_weight.is_none() {
                typo.font_weight = self.font_weight.clone();
            }
            if typo.text_align.is_none() {
                typo.text_align = self.text_align.clone();
            }
            if typo.line_height.is_none() {
                typo.line_height = self.line_height.clone();
            }
        }

        // 3. Padding
        if let Some(ref pad) = self.padding {
            if self.padding_top.is_none() {
                self.padding_top = pad.top.clone();
            }
            if self.padding_right.is_none() {
                self.padding_right = pad.right.clone();
            }
            if self.padding_bottom.is_none() {
                self.padding_bottom = pad.bottom.clone();
            }
            if self.padding_left.is_none() {
                self.padding_left = pad.left.clone();
            }
        }
        if self.padding_top.is_some()
            || self.padding_right.is_some()
            || self.padding_bottom.is_some()
            || self.padding_left.is_some()
        {
            let pad = self.padding.get_or_insert_with(Default::default);
            if pad.top.is_none() {
                pad.top = self.padding_top.clone();
            }
            if pad.right.is_none() {
                pad.right = self.padding_right.clone();
            }
            if pad.bottom.is_none() {
                pad.bottom = self.padding_bottom.clone();
            }
            if pad.left.is_none() {
                pad.left = self.padding_left.clone();
            }
        }

        // 4. Margin
        if let Some(ref mar) = self.margin {
            if self.margin_top.is_none() {
                self.margin_top = mar.top.clone();
            }
            if self.margin_right.is_none() {
                self.margin_right = mar.right.clone();
            }
            if self.margin_bottom.is_none() {
                self.margin_bottom = mar.bottom.clone();
            }
            if self.margin_left.is_none() {
                self.margin_left = mar.left.clone();
            }
        }
        if self.margin_top.is_some()
            || self.margin_right.is_some()
            || self.margin_bottom.is_some()
            || self.margin_left.is_some()
        {
            let mar = self.margin.get_or_insert_with(Default::default);
            if mar.top.is_none() {
                mar.top = self.margin_top.clone();
            }
            if mar.right.is_none() {
                mar.right = self.margin_right.clone();
            }
            if mar.bottom.is_none() {
                mar.bottom = self.margin_bottom.clone();
            }
            if mar.left.is_none() {
                mar.left = self.margin_left.clone();
            }
        }

        // 5. Border radius
        if let Some(ref bdr) = self.border {
            if self.border_radius.is_none() {
                self.border_radius = bdr.radius.clone();
            }
        }
        if let Some(ref rad) = self.border_radius {
            let bdr = self.border.get_or_insert_with(Default::default);
            if bdr.radius.is_none() {
                bdr.radius = Some(rad.clone());
            }
        }

        // 6. Recursive descent into tablet, mobile, and hover overrides
        if let Some(ref mut tab) = self.tablet {
            tab.sync_flat_and_nested();
        }
        if let Some(ref mut mob) = self.mobile {
            mob.sync_flat_and_nested();
        }
        if let Some(ref mut hov) = self.hover {
            hov.sync_flat_and_nested();
        }
    }
}
