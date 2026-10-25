use crate::render::sanitizer::escape_attr;

pub(crate) fn render_form_html(
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
        "<form id=\"{}\" class=\"np-search-form\" method=\"GET\" action=\"{}\"{}>\n<div class=\"np-search-input-wrap\">\n<input type=\"search\" name=\"s\" class=\"np-search-input\" placeholder=\"{}\" required />\n<button type=\"submit\" class=\"np-search-btn\">{}</button>\n</div>\n</form>",
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
        "<label class=\"np-login-remember\"><input type=\"checkbox\" name=\"rememberme\" value=\"forever\" /> Remember Me</label>\n"
    } else {
        ""
    };
    format!(
        "<form id=\"{}\" class=\"np-login-form\" method=\"POST\" action=\"/login\"{}>\n<div class=\"np-form-field\">\n<label>{}</label>\n<input type=\"text\" name=\"log\" class=\"np-input\" required autocomplete=\"username\" />\n</div>\n<div class=\"np-form-field\">\n<label>{}</label>\n<input type=\"password\" name=\"pwd\" class=\"np-input\" required autocomplete=\"current-password\" />\n</div>\n{}<input type=\"hidden\" name=\"redirect_to\" value=\"{}\" />\n<button type=\"submit\" class=\"np-button np-login-btn\">Log In</button>\n</form>",
        escape_attr(node_id),
        style_attr,
        escape_attr(label_user),
        escape_attr(label_pass),
        rem_html,
        escape_attr(redirect_url)
    )
}
