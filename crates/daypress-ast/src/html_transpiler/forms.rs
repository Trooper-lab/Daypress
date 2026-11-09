//! HTML `<form>` → FormStudio form specification (`html_transpiler/forms.rs`).
//!
//! An imported `<form>` becomes a native `Form` node that keeps the design's own field markup,
//! plus an [`ImportedForm`] that the server saves as a FormStudio form (so submissions land in the
//! Submissions Vault). Field names, labels, `required`, placeholders, select options, checkbox
//! groups (`checkbox_group`), radios and file inputs (`file`, with the `accept` list) are read
//! from the markup.

use super::dom_parser::{DomElement, DomNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A form found in an imported design, ready to be saved as a FormStudio form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportedForm {
    /// Negative placeholder carried by the page's `Form` node (`form_id`) until the server has
    /// saved the form and swapped in the real id: -1 for the first form, -2 for the second, ...
    pub placeholder_id: i64,
    /// Short human name: the form's `aria-label` / `title`, else its submit label, else the
    /// hidden `_subject`, else "Form".
    pub label: String,
    /// Fields in `FormFieldSchema` JSON shape.
    pub fields: Vec<ImportedFormField>,
    pub submit_label: Option<String>,
    /// Text of the design's (hidden) success / status message.
    pub success_message: Option<String>,
    /// Value of a hidden `_subject` input (Formspree convention), used as the e-mail subject.
    pub email_subject: Option<String>,
}

/// One field, serialized with the same keys as the server's `FormFieldSchema`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportedFormField {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub placeholder: String,
    pub options: Vec<String>,
    pub width: String,
    pub step: u8,
    pub default_value: String,
    pub accept_extensions: String,
    pub max_file_size_mb: u32,
}

impl ImportedFormField {
    fn new(name: &str, label: String, field_type: &str) -> Self {
        Self {
            name: name.to_string(),
            label,
            field_type: field_type.to_string(),
            required: false,
            placeholder: String::new(),
            options: Vec::new(),
            width: "full".to_string(),
            step: 1,
            default_value: String::new(),
            accept_extensions: String::new(),
            max_file_size_mb: 10,
        }
    }
}

/// Tags that carry a value FormStudio stores.
pub fn is_form_control_tag(tag: &str) -> bool {
    matches!(tag, "input" | "select" | "textarea")
}

/// A spam trap input (`_gotcha`, `honeypot`, hidden from people via inline style or ARIA).
pub fn is_honeypot(el: &DomElement) -> bool {
    if el.tag != "input" {
        return false;
    }
    let name = el.attr("name").unwrap_or("").to_ascii_lowercase();
    let style = el
        .attr("style")
        .unwrap_or("")
        .to_ascii_lowercase()
        .replace(' ', "");
    name.contains("gotcha")
        || name.contains("honeypot")
        || style.contains("display:none")
        || style.contains("visibility:hidden")
        || (el.attr("aria-hidden") == Some("true") && el.attr("tabindex") == Some("-1"))
}

/// Hidden inputs FormStudio does not need: third-party service settings such as Formspree's
/// `_subject`, `_next`, `_replyto`, `_cc`, and honeypots.
pub fn is_service_hidden_input(el: &DomElement) -> bool {
    if el.tag != "input" {
        return false;
    }
    let ty = el.attr("type").unwrap_or("text").to_ascii_lowercase();
    is_honeypot(el) || (ty == "hidden" && el.attr("name").unwrap_or("").starts_with('_'))
}

/// `true` for `<button>` / `<input>` elements that submit their form.
pub fn is_submit_control(el: &DomElement) -> bool {
    let ty = el.attr("type").map(str::to_ascii_lowercase);
    match el.tag.as_str() {
        "button" => matches!(ty.as_deref(), None | Some("submit") | Some("")),
        "input" => matches!(ty.as_deref(), Some("submit") | Some("image")),
        _ => false,
    }
}

/// `<input>` elements drawn as buttons (an `image` input is a picture, not a label).
pub fn is_input_button(el: &DomElement) -> bool {
    el.tag == "input"
        && matches!(
            el.attr("type").map(str::to_ascii_lowercase).as_deref(),
            Some("submit" | "button" | "reset")
        )
}

/// FormStudio field name for a control: `name` without a trailing `[]`, else the `id`.
pub fn field_name(el: &DomElement) -> Option<String> {
    el.attr("name")
        .or_else(|| el.id())
        .map(|n| n.trim().trim_end_matches("[]").trim().to_string())
        .filter(|n| !n.is_empty())
}

/// Reads a form's FormStudio specification.
pub fn extract_form(form: &DomElement, placeholder_id: i64) -> ImportedForm {
    let mut labels_for: HashMap<String, String> = HashMap::new();
    collect_for_labels(form, &mut labels_for);
    let has_full_marker = any_descendant(form, &|e| has_class_token(e, "full"));

    let mut ex = Extractor {
        labels_for,
        has_full_marker,
        fields: Vec::new(),
        submit_label: None,
        success_message: None,
        email_subject: None,
    };
    let mut ctx = WalkCtx::default();
    ex.walk(form, &mut ctx);

    let label = [
        form.attr("aria-label"),
        form.attr("title"),
        ex.submit_label.as_deref(),
        ex.email_subject.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(clean_text)
    .find(|s| !s.is_empty())
    .unwrap_or_else(|| "Form".to_string());

    ImportedForm {
        placeholder_id,
        label,
        fields: ex.fields,
        submit_label: ex.submit_label,
        success_message: ex.success_message,
        email_subject: ex.email_subject,
    }
}

#[derive(Default, Clone)]
struct WalkCtx {
    /// Text of the nearest wrapping `<label>`.
    wrapping_label: Option<String>,
    /// Text of the nearest `<fieldset>`'s `<legend>`.
    legend: Option<String>,
    /// Inside an element with a `full` class token (a full-width field).
    in_full: bool,
}

struct Extractor {
    labels_for: HashMap<String, String>,
    has_full_marker: bool,
    fields: Vec<ImportedFormField>,
    submit_label: Option<String>,
    success_message: Option<String>,
    email_subject: Option<String>,
}

impl Extractor {
    fn walk(&mut self, el: &DomElement, ctx: &mut WalkCtx) {
        for child in &el.children {
            let DomNode::Element(c) = child else {
                continue;
            };
            match c.tag.as_str() {
                "input" => self.input(c, ctx),
                "select" => self.select(c, ctx),
                "textarea" => self.textarea(c, ctx),
                "button" => {
                    if is_submit_control(c) && self.submit_label.is_none() {
                        let text = clean_text(&c.text_content());
                        if !text.is_empty() {
                            self.submit_label = Some(text);
                        }
                    }
                }
                _ => {
                    if self.success_message.is_none() && is_status_element(c) {
                        let text = clean_text(&c.text_content());
                        if !text.is_empty() {
                            self.success_message = Some(text);
                        }
                        continue;
                    }
                    let mut inner = ctx.clone();
                    if c.tag == "label" {
                        inner.wrapping_label = Some(label_text(c));
                    }
                    if c.tag == "fieldset" {
                        inner.legend = c.children.iter().find_map(|n| match n {
                            DomNode::Element(e) if e.tag == "legend" => {
                                Some(clean_text(&e.text_content()))
                            }
                            _ => None,
                        });
                    }
                    if has_class_token(c, "full") {
                        inner.in_full = true;
                    }
                    self.walk(c, &mut inner);
                }
            }
        }
    }

    fn label_for(&self, el: &DomElement, ctx: &WalkCtx) -> Option<String> {
        el.id()
            .and_then(|id| self.labels_for.get(id).cloned())
            .or_else(|| ctx.wrapping_label.clone())
            .or_else(|| el.attr("aria-label").map(clean_text))
            .filter(|s| !s.is_empty())
    }

    fn width(&self, ctx: &WalkCtx, field_type: &str) -> String {
        if !self.has_full_marker
            || ctx.in_full
            || matches!(field_type, "textarea" | "checkbox_group" | "radio")
        {
            "full".to_string()
        } else {
            "half".to_string()
        }
    }

    fn input(&mut self, el: &DomElement, ctx: &WalkCtx) {
        let ty = el
            .attr("type")
            .unwrap_or("text")
            .trim()
            .to_ascii_lowercase();
        if is_submit_control(el) {
            if self.submit_label.is_none() {
                let text = clean_text(el.attr("value").unwrap_or(""));
                if !text.is_empty() {
                    self.submit_label = Some(text);
                }
            }
            return;
        }
        if matches!(ty.as_str(), "button" | "reset") || is_honeypot(el) {
            return;
        }
        let Some(name) = field_name(el) else {
            return;
        };
        if ty == "hidden" {
            if name.eq_ignore_ascii_case("_subject") {
                let subject = clean_text(el.attr("value").unwrap_or(""));
                if !subject.is_empty() {
                    self.email_subject = Some(subject);
                }
                return;
            }
            if name.starts_with('_') {
                return;
            }
            let mut f = ImportedFormField::new(&name, humanize(&name), "hidden");
            f.default_value = el.attr("value").unwrap_or("").to_string();
            self.push(f);
            return;
        }

        let raw_name = el.attr("name").unwrap_or("");
        let required = el.attr("required").is_some();
        if ty == "checkbox" || ty == "radio" {
            let option = el
                .attr("value")
                .map(clean_text)
                .filter(|v| !v.is_empty() && v != "on")
                .or_else(|| ctx.wrapping_label.clone())
                .or_else(|| el.id().and_then(|id| self.labels_for.get(id).cloned()))
                .unwrap_or_else(|| "yes".to_string());
            if let Some(existing) = self.fields.iter_mut().find(|f| f.name == name) {
                if ty == "checkbox" && existing.field_type == "checkbox" {
                    existing.field_type = "checkbox_group".to_string();
                    if let Some(legend) = ctx.legend.clone() {
                        existing.label = legend;
                    }
                }
                if !existing.options.contains(&option) {
                    existing.options.push(option);
                }
                existing.required |= required && ty == "radio";
                return;
            }
            let is_group = ty == "radio" || raw_name.trim_end().ends_with("[]");
            let field_type = match (ty.as_str(), is_group) {
                ("radio", _) => "radio",
                (_, true) => "checkbox_group",
                _ => "checkbox",
            };
            let label = if field_type == "checkbox" {
                ctx.wrapping_label
                    .clone()
                    .or_else(|| self.label_for(el, ctx))
                    .unwrap_or_else(|| humanize(&name))
            } else {
                ctx.legend.clone().unwrap_or_else(|| humanize(&name))
            };
            let mut f = ImportedFormField::new(&name, label, field_type);
            f.required = required;
            f.options.push(option);
            f.width = self.width(ctx, field_type);
            self.push(f);
            return;
        }

        let field_type = match ty.as_str() {
            "email" | "tel" | "url" | "number" | "date" | "time" | "file" => ty.as_str(),
            "range" => "number",
            "datetime-local" | "month" | "week" => "date",
            _ => "text",
        };
        let label = self
            .label_for(el, ctx)
            .or_else(|| el.attr("placeholder").map(clean_text))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| humanize(&name));
        let mut f = ImportedFormField::new(&name, label, field_type);
        f.required = required;
        f.placeholder = clean_text(el.attr("placeholder").unwrap_or(""));
        f.default_value = if field_type == "file" {
            String::new()
        } else {
            el.attr("value").unwrap_or("").to_string()
        };
        if field_type == "file" {
            f.accept_extensions = accept_to_extensions(el.attr("accept").unwrap_or(""));
        }
        f.width = self.width(ctx, field_type);
        self.push(f);
    }

    fn select(&mut self, el: &DomElement, ctx: &WalkCtx) {
        let Some(name) = field_name(el) else {
            return;
        };
        let mut placeholder = String::new();
        let mut options = Vec::new();
        collect_options(el, &mut |opt: &DomElement| {
            let text = clean_text(&opt.text_content());
            let value = opt.attr("value").map(clean_text);
            if value.as_deref() == Some("") && options.is_empty() && placeholder.is_empty() {
                placeholder = text;
                return;
            }
            let v = value.filter(|v| !v.is_empty()).unwrap_or(text);
            if !v.is_empty() && !options.contains(&v) {
                options.push(v);
            }
        });
        let multiple = el.attr("multiple").is_some();
        let field_type = if multiple { "checkbox_group" } else { "select" };
        let label = self.label_for(el, ctx).unwrap_or_else(|| humanize(&name));
        let mut f = ImportedFormField::new(&name, label, field_type);
        f.required = el.attr("required").is_some();
        f.placeholder = placeholder;
        f.options = options;
        f.width = self.width(ctx, field_type);
        self.push(f);
    }

    fn textarea(&mut self, el: &DomElement, ctx: &WalkCtx) {
        let Some(name) = field_name(el) else {
            return;
        };
        let label = self
            .label_for(el, ctx)
            .or_else(|| el.attr("placeholder").map(clean_text))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| humanize(&name));
        let mut f = ImportedFormField::new(&name, label, "textarea");
        f.required = el.attr("required").is_some();
        f.placeholder = clean_text(el.attr("placeholder").unwrap_or(""));
        f.default_value = decode_entities(&el.text_content());
        f.width = self.width(ctx, "textarea");
        self.push(f);
    }

    fn push(&mut self, f: ImportedFormField) {
        if !self.fields.iter().any(|e| e.name == f.name) {
            self.fields.push(f);
        }
    }
}

/// The design's success / status message: hidden until a script reveals it.
pub fn is_status_element(el: &DomElement) -> bool {
    let role_status = matches!(el.attr("role"), Some("status") | Some("alert"));
    let success_class = el
        .classes()
        .iter()
        .any(|c| c.to_ascii_lowercase().contains("success"));
    (el.attr("hidden").is_some() && (role_status || success_class))
        || (success_class && role_status)
}

fn collect_for_labels(el: &DomElement, out: &mut HashMap<String, String>) {
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if c.tag == "label" {
                if let Some(target) = c.attr("for").filter(|s| !s.trim().is_empty()) {
                    out.insert(target.trim().to_string(), label_text(c));
                }
            }
            collect_for_labels(c, out);
        }
    }
}

/// Label text without nested controls and without required-marker asterisks.
fn label_text(label: &DomElement) -> String {
    fn text_skipping_controls(el: &DomElement, out: &mut String) {
        for child in &el.children {
            match child {
                DomNode::Text(t) => out.push_str(t),
                DomNode::Element(c) if !is_form_control_tag(&c.tag) => {
                    text_skipping_controls(c, out)
                }
                _ => {}
            }
        }
    }
    let mut raw = String::new();
    text_skipping_controls(label, &mut raw);
    clean_text(&raw)
}

fn collect_options(el: &DomElement, f: &mut dyn FnMut(&DomElement)) {
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if c.tag == "option" {
                f(c);
            } else if c.tag == "optgroup" {
                collect_options(c, f);
            }
        }
    }
}

fn any_descendant(el: &DomElement, pred: &dyn Fn(&DomElement) -> bool) -> bool {
    el.children.iter().any(|n| match n {
        DomNode::Element(c) => pred(c) || any_descendant(c, pred),
        DomNode::Text(_) => false,
    })
}

fn has_class_token(el: &DomElement, token: &str) -> bool {
    el.classes().contains(&token)
}

/// Converts an `accept` attribute (`image/*,.pdf`) into FormStudio's extension allowlist.
pub fn accept_to_extensions(accept: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut push = |e: &'static str| {
        if !out.contains(&e) {
            out.push(e);
        }
    };
    let mut extra: Vec<String> = Vec::new();
    for part in accept.split(',').map(|p| p.trim().to_ascii_lowercase()) {
        match part.as_str() {
            "" => {}
            "image/*" => ["jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "avif"]
                .into_iter()
                .for_each(&mut push),
            "application/pdf" => push("pdf"),
            "image/jpeg" | "image/jpg" => {
                push("jpg");
                push("jpeg")
            }
            "image/png" => push("png"),
            "image/webp" => push("webp"),
            "image/gif" => push("gif"),
            "image/heic" => push("heic"),
            "video/*" => ["mp4", "mov", "webm"].into_iter().for_each(&mut push),
            p if p.starts_with('.') => {
                let ext = p.trim_start_matches('.').to_string();
                if !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
                    extra.push(ext);
                }
            }
            _ => {}
        }
    }
    let mut all: Vec<String> = out.into_iter().map(String::from).collect();
    for e in extra {
        if !all.contains(&e) {
            all.push(e);
        }
    }
    all.join(",")
}

/// Collapses whitespace, decodes basic entities, and drops required-marker asterisks and a
/// trailing colon.
pub fn clean_text(raw: &str) -> String {
    let decoded = decode_entities(raw);
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim()
        .trim_end_matches(['*', ':', ' '])
        .trim()
        .to_string()
}

/// Decodes the handful of HTML entities that show up in labels and options.
pub fn decode_entities(raw: &str) -> String {
    raw.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// `bedrijf_naam` / `bedrijf-naam` → `Bedrijf naam`.
fn humanize(name: &str) -> String {
    let spaced: String = name
        .chars()
        .map(|c| if c == '_' || c == '-' { ' ' } else { c })
        .collect();
    let trimmed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Serializes the children of `el` for a raw field block: drops service hidden inputs and
/// honeypots and strips `[]` from control names so they match the FormStudio field names.
pub fn field_markup(el: &DomElement) -> String {
    let mut out = String::new();
    for child in &el.children {
        if let Some(n) = normalize_node(child) {
            n.write_html(&mut out);
        }
    }
    out.trim().to_string()
}

/// Serializes a single control (and its subtree) the same way as [`field_markup`].
pub fn control_markup(el: &DomElement) -> String {
    let mut out = String::new();
    if let Some(n) = normalize_node(&DomNode::Element(el.clone())) {
        n.write_html(&mut out);
    }
    out
}

fn normalize_node(node: &DomNode) -> Option<DomNode> {
    match node {
        DomNode::Text(t) => Some(DomNode::Text(t.clone())),
        DomNode::Element(el) => {
            if is_service_hidden_input(el) {
                return None;
            }
            let mut copy = el.clone();
            if is_form_control_tag(&copy.tag) {
                if let Some(name) = copy.attrs.get_mut("name") {
                    let trimmed = name.trim().trim_end_matches("[]").to_string();
                    *name = trimmed;
                }
            }
            copy.children = el.children.iter().filter_map(normalize_node).collect();
            Some(DomNode::Element(copy))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_transpiler::dom_parser::parse_html_dom;

    fn first_form(html: &str) -> DomElement {
        fn find(el: &DomElement) -> Option<DomElement> {
            if el.tag == "form" {
                return Some(el.clone());
            }
            el.children.iter().find_map(|c| match c {
                DomNode::Element(e) => find(e),
                _ => None,
            })
        }
        find(&parse_html_dom(html)).expect("form in fixture")
    }

    #[test]
    fn reads_fields_options_groups_and_messages() {
        let form = first_form(
            r#"<form action="https://formspree.io/f/x" method="POST">
              <input type="hidden" name="_subject" value="Sample request">
              <input type="hidden" name="source" value="Landing">
              <input type="text" name="_gotcha" style="display:none" tabindex="-1">
              <div class="grid">
                <div class="field"><label for="n">Naam <span class="req">*</span></label><input type="text" id="n" name="naam" required></div>
                <div class="field full"><label for="t">Ik ben</label>
                  <select id="t" name="type" required><option value="">Maak een keuze</option><option>Architect</option><option>Anders &amp; meer</option></select></div>
                <fieldset class="field full"><legend>Toepassing</legend>
                  <label class="chip"><input type="checkbox" name="toepassing[]" value="Keuken"><span>Keuken</span></label>
                  <label class="chip"><input type="checkbox" name="toepassing[]" value="Deur"><span>Deur</span></label>
                </fieldset>
                <div class="field full"><label for="f">Foto</label><input type="file" id="f" name="bijlage" accept="image/*,.pdf"></div>
                <div class="field full"><label for="m">Vertel</label><textarea id="m" name="toelichting" placeholder="Bijvoorbeeld"></textarea></div>
                <label><input type="checkbox" name="akkoord" required> Ik ga akkoord</label>
              </div>
              <button type="submit" class="btn">Verstuur aanvraag</button>
              <div class="form-success" role="status" hidden>Dank je wel.</div>
            </form>"#,
        );
        let spec = extract_form(&form, -1);
        assert_eq!(spec.placeholder_id, -1);
        assert_eq!(spec.label, "Verstuur aanvraag");
        assert_eq!(spec.submit_label.as_deref(), Some("Verstuur aanvraag"));
        assert_eq!(spec.success_message.as_deref(), Some("Dank je wel."));
        assert_eq!(spec.email_subject.as_deref(), Some("Sample request"));

        let by = |n: &str| spec.fields.iter().find(|f| f.name == n).cloned();
        assert!(by("_gotcha").is_none() && by("_subject").is_none());

        let source = by("source").expect("hidden field");
        assert_eq!(source.field_type, "hidden");
        assert_eq!(source.default_value, "Landing");

        let naam = by("naam").expect("naam");
        assert_eq!(
            (naam.label.as_str(), naam.field_type.as_str()),
            ("Naam", "text")
        );
        assert!(naam.required);
        assert_eq!(naam.width, "half");

        let ty = by("type").expect("select");
        assert_eq!(ty.field_type, "select");
        assert_eq!(ty.placeholder, "Maak een keuze");
        assert_eq!(ty.options, vec!["Architect", "Anders & meer"]);
        assert_eq!(ty.width, "full");

        let group = by("toepassing").expect("checkbox group");
        assert_eq!(group.field_type, "checkbox_group");
        assert_eq!(group.label, "Toepassing");
        assert_eq!(group.options, vec!["Keuken", "Deur"]);

        let file = by("bijlage").expect("file");
        assert_eq!(file.field_type, "file");
        assert!(file.accept_extensions.split(',').any(|e| e == "jpg"));
        assert!(file.accept_extensions.split(',').any(|e| e == "pdf"));

        let msg = by("toelichting").expect("textarea");
        assert_eq!(msg.field_type, "textarea");
        assert_eq!(msg.placeholder, "Bijvoorbeeld");

        let consent = by("akkoord").expect("single checkbox");
        assert_eq!(consent.field_type, "checkbox");
        assert_eq!(consent.label, "Ik ga akkoord");
        assert!(consent.required);
    }

    #[test]
    fn field_markup_keeps_attributes_and_normalizes_names() {
        let form = first_form(
            r#"<form><label class="chip"><input type="checkbox" name="metalen[]" value="Brons"><span>Brons</span></label><input type="hidden" name="_next" value="/x"></form>"#,
        );
        let html = field_markup(&form);
        assert!(html.contains(r#"name="metalen""#), "{html}");
        assert!(html.contains(r#"value="Brons""#));
        assert!(!html.contains("_next"));
    }

    #[test]
    fn accept_lists_become_extensions() {
        assert_eq!(accept_to_extensions(".pdf, .DOCX"), "pdf,docx");
        assert_eq!(accept_to_extensions("application/pdf,image/png"), "pdf,png");
        assert_eq!(accept_to_extensions(""), "");
    }
}
