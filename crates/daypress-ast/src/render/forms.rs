use crate::render::sanitizer::escape_attr;

/// Submit handler for AST `Form` nodes: posts to FormStudio (multipart when a file input is
/// present, JSON otherwise, with repeated names such as checkbox groups joined by ", "), then
/// reveals the `.dp-form-status` line with the server's message.
const FORM_SUBMIT_JS: &str = "event.preventDefault();var f=this,s=f.querySelector('.dp-form-status'),b=f.querySelector('[type=submit]'),fd=new FormData(f),hasFile=!!f.querySelector('input[type=file]'),o={};fd.forEach(function(v,k){o[k]=Object.prototype.hasOwnProperty.call(o,k)?o[k]+', '+v:v;});var reqInit=hasFile?{method:'POST',body:fd}:{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(o)};if(b)b.disabled=true;fetch(f.action,reqInit).then(function(r){return r.json().then(function(j){return {ok:r.ok,j:j};}).catch(function(){return {ok:r.ok,j:{}};});}).catch(function(){return {ok:false,j:{}};}).then(function(res){if(b)b.disabled=false;var okMsg=(res.j&&res.j.message)||(s&&s.getAttribute('data-success-msg'))||'Thank you! Your message has been sent.';var redir=(res.j&&res.j.redirect_url)||(s&&s.getAttribute('data-redirect-url'));if(s){s.hidden=false;s.style.display='';s.textContent=res.ok?okMsg:((res.j&&res.j.message)||'Submission failed.');}if(res.ok){f.reset();if(redir)window.location.href=redir;}});return false;";

pub(crate) fn render_form_html(
    node_id: &str,
    form_id: i64,
    title: Option<&str>,
    button_text: Option<&str>,
    endpoint: Option<&str>,
    style_attr: &str,
    inner_html: &str,
) -> String {
    render_form_html_with_classes(
        node_id,
        form_id,
        title,
        button_text,
        endpoint,
        style_attr,
        inner_html,
        None,
    )
}

/// Same as [`render_form_html`], adding the node's own classes when the form renders its child
/// blocks (an imported design's form markup). Placeholder forms keep the exact
/// `class="dp-form"` the server hydrates with the FormStudio form.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_form_html_with_classes(
    node_id: &str,
    form_id: i64,
    title: Option<&str>,
    button_text: Option<&str>,
    endpoint: Option<&str>,
    style_attr: &str,
    inner_html: &str,
    extra_cls: Option<&str>,
) -> String {
    let action = endpoint
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("/wp-json/daypress/v1/forms/{}/submit", form_id));

    let title_html = title
        .filter(|t| !t.trim().is_empty())
        .map(|t| format!("<h3 class=\"dp-form-title\">{}</h3>\n", escape_attr(t)))
        .unwrap_or_default();

    let btn_label = button_text
        .filter(|b| !b.trim().is_empty())
        .unwrap_or("Send Message");

    let has_children = !inner_html.trim().is_empty();
    let fields_html = if !has_children {
        format!(
            "<div class=\"dp-form-fields\">\n<input type=\"text\" name=\"name\" class=\"dp-input\" placeholder=\"Your Name\" required />\n<input type=\"email\" name=\"email\" class=\"dp-input\" placeholder=\"Email Address\" required />\n<textarea name=\"message\" class=\"dp-textarea\" rows=\"4\" placeholder=\"Your Message\" required></textarea>\n<button type=\"submit\" class=\"dp-button dp-form-submit\">{}</button>\n</div>\n<div class=\"dp-form-status\" aria-live=\"polite\"></div>",
            escape_attr(btn_label)
        )
    } else if inner_html.contains("dp-form-status") {
        inner_html.to_string()
    } else {
        format!(
            "{}\n<div class=\"dp-form-status\" role=\"status\" aria-live=\"polite\" hidden></div>",
            inner_html
        )
    };

    // Forms that render their own field blocks get `dp-form-markup`, so the server never mistakes
    // them for a placeholder form to hydrate.
    let class_attr = if has_children {
        match extra_cls.map(str::trim).filter(|c| !c.is_empty()) {
            Some(extra) => format!("dp-form dp-form-markup {}", escape_attr(extra)),
            None => "dp-form dp-form-markup".to_string(),
        }
    } else {
        "dp-form".to_string()
    };

    format!(
        "<form id=\"{}\" class=\"{}\" data-form-id=\"{}\" method=\"POST\" action=\"{}\" onsubmit=\"{}\"{}>\n{}<input type=\"text\" name=\"_np_hp\" style=\"display:none\" tabindex=\"-1\" autocomplete=\"off\" />\n{}\n</form>",
        escape_attr(node_id),
        class_attr,
        form_id,
        escape_attr(&action),
        FORM_SUBMIT_JS,
        style_attr,
        title_html,
        fields_html
    )
}

pub(crate) fn render_search_form_html(
    node_id: &str,
    placeholder: &str,
    button_text: &str,
    endpoint: &str,
    style_attr: &str,
) -> String {
    let action = if endpoint.trim().is_empty() {
        "/"
    } else {
        endpoint.trim()
    };
    format!(
        "<form id=\"{}\" class=\"dp-search-form\" method=\"GET\" action=\"{}\"{}>\n<div class=\"dp-search-input-wrap\">\n<input type=\"search\" name=\"s\" class=\"dp-search-input\" placeholder=\"{}\" required />\n<button type=\"submit\" class=\"dp-search-btn\">{}</button>\n</div>\n</form>",
        escape_attr(node_id),
        escape_attr(action),
        style_attr,
        escape_attr(placeholder),
        escape_attr(button_text),
    )
}

pub(crate) fn render_login_form_html(
    node_id: &str,
    label_user: &str,
    label_pass: &str,
    redirect_url: &str,
    remember_me: bool,
    style_attr: &str,
) -> String {
    let rem_html = if remember_me {
        "<label class=\"dp-login-remember\"><input type=\"checkbox\" name=\"rememberme\" value=\"forever\" /> Remember Me</label>\n"
    } else {
        ""
    };
    format!(
        "<form id=\"{}\" class=\"dp-login-form\" method=\"POST\" action=\"/login\"{}>\n<div class=\"dp-form-field\">\n<label>{}</label>\n<input type=\"text\" name=\"log\" class=\"dp-input\" required autocomplete=\"username\" />\n</div>\n<div class=\"dp-form-field\">\n<label>{}</label>\n<input type=\"password\" name=\"pwd\" class=\"dp-input\" required autocomplete=\"current-password\" />\n</div>\n{}<input type=\"hidden\" name=\"redirect_to\" value=\"{}\" />\n<button type=\"submit\" class=\"dp-button dp-login-btn\">Log In</button>\n</form>",
        escape_attr(node_id),
        style_attr,
        escape_attr(label_user),
        escape_attr(label_pass),
        rem_html,
        escape_attr(redirect_url)
    )
}

#[cfg(test)]
mod tests {
    use crate::schema::{DaypressNode, NodeKind, StyleProperties};

    fn form_node(children: Vec<DaypressNode>, classes: Option<&str>) -> DaypressNode {
        DaypressNode {
            id: "f1".into(),
            kind: NodeKind::Form {
                form_id: 7,
                title: None,
                button_text: Some("Verstuur".into()),
                endpoint: None,
            },
            styles: StyleProperties {
                css_classes: classes.map(String::from),
                ..Default::default()
            },
            children,
        }
    }

    fn submit_button() -> DaypressNode {
        DaypressNode {
            id: "b1".into(),
            kind: NodeKind::Button {
                text: "Verstuur".into(),
                url: String::new(),
                target: "submit".into(),
            },
            styles: StyleProperties {
                html_tag: Some("button".into()),
                css_classes: Some("btn".into()),
                ..Default::default()
            },
            children: Vec::new(),
        }
    }

    #[test]
    fn form_with_design_markup_keeps_classes_and_gets_a_status_line() {
        let html = crate::render_node_to_html(&form_node(vec![submit_button()], Some("js-form")));
        assert!(
            html.contains(r#"class="dp-form dp-form-markup js-form" data-form-id="7""#),
            "{html}"
        );
        assert!(html.contains(r#"action="/wp-json/daypress/v1/forms/7/submit""#));
        assert!(html
            .contains(r#"<button type="submit" class="dp-button btn" id="b1">Verstuur</button>"#));
        assert!(html.contains(
            r#"<div class="dp-form-status" role="status" aria-live="polite" hidden></div>"#
        ));
        // Repeated names (checkbox groups) are joined instead of keeping only the last value.
        assert!(html.contains("hasOwnProperty.call(o,k)?o[k]+', '+v:v"));
        assert!(!html.contains("Object.fromEntries"));
    }

    #[test]
    fn placeholder_form_keeps_the_hydration_markup() {
        let html = crate::render_node_to_html(&form_node(Vec::new(), Some("js-form")));
        assert!(
            html.contains(r#"class="dp-form" data-form-id="7""#),
            "{html}"
        );
        assert!(html.contains(r#"<div class="dp-form-fields">"#));
    }

    #[test]
    fn plain_buttons_stay_type_button() {
        let mut b = submit_button();
        b.kind = NodeKind::Button {
            text: "Menu".into(),
            url: String::new(),
            target: "_self".into(),
        };
        assert!(crate::render_node_to_html(&b).starts_with(r#"<button type="button""#));
    }
}
