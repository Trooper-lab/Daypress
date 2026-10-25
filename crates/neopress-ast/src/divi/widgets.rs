use crate::divi::context::SpbServiceItem;
use crate::divi::parser::parse_attributes;
use regex::Regex;
use std::collections::HashMap;

pub(crate) fn escape_html(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn expand_cf7_shortcodes_in_html(
    html: &str,
    cf7_forms: &HashMap<String, String>,
) -> String {
    let re = Regex::new(r"\[contact-form-7([^\]]*)\]").unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let attr_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let attrs = parse_attributes(attr_str);
        if let Some(id_val) = attrs.get("id") {
            if let Some(form_html) = cf7_forms.get(id_val) {
                return form_html.clone();
            }
            let prefix: String = id_val.chars().take(7).collect();
            if let Some(form_html) = cf7_forms.get(&prefix) {
                return form_html.clone();
            }
        }
        if let Some(title_val) = attrs.get("title") {
            if let Some(form_html) = cf7_forms.get(&title_val.to_lowercase()) {
                return form_html.clone();
            }
        }
        if let Some((_, first_form)) = cf7_forms.iter().next() {
            return first_form.clone();
        }
        caps.get(0).unwrap().as_str().to_string()
    })
    .to_string()
}

pub(crate) fn render_fallback_cf7_form(form_id: &str) -> String {
    format!(
        r#"<form class="wpcf7-form np-cf7-form np-form" method="POST" action="/wp-json/neopress/v1/forms/1/submit" onsubmit="event.preventDefault();var f=this,s=f.querySelector('.np-form-status'),d=Object.fromEntries(new FormData(f).entries());fetch(f.action,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(d)}}).then(function(r){{if(s)s.textContent=r.ok?'Bedankt voor je bericht. Het is verzonden.':'Er is een fout opgetreden.';if(r.ok)f.reset();}});return false;" data-cf7-id="{}">
<div class="dipe-form-row" style="display:flex;gap:16px;flex-wrap:wrap;margin-bottom:12px;">
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Wat is je naam?<input type="text" name="your-name" placeholder="Voor- en achternaam" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Waar ontvang je e-post?<input type="email" name="your-email" placeholder="Email-adres" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
</div>
<div class="dipe-form-row" style="display:flex;gap:16px;flex-wrap:wrap;margin-bottom:12px;">
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Mogen we je appen?*<input type="tel" name="tel-604" placeholder="Whatsapp nummer" style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Is t ver weg?<input type="text" name="your-plaats" placeholder="Woonplaats" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
</div>
<label style="color:#fff;display:block;margin-bottom:12px;">Wat moet er gekit worden?:<textarea name="your-message" rows="4" placeholder="Duidelijke opdrachtomschriving" style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;"></textarea></label>
<div style="text-align:right;"><button type="submit" class="et_pb_button" style="background:#e23212;color:#fcfbfc;border:2px solid #fff;border-radius:8px;padding:10px 24px;font-size:1.4em;cursor:pointer;">Verzenden</button></div>
<div class="np-form-status" aria-live="polite" style="color:#7CDA24;margin-top:12px;font-weight:600;"></div>
</form>"#,
        escape_html(form_id)
    )
}

/// Renders the complete interactive 2-step service price calculator (`[spb_kw_calculator_module]` & `[spb_kitwerk_calculator]`)
/// populated with the `spb_service` items imported from WXR.
pub fn render_spb_calculator_widget(services: &[SpbServiceItem]) -> String {
    let default_services = vec![
        SpbServiceItem {
            id: 583,
            title: "Keuken/Badkamer/sanitair".to_string(),
            price: 9.0,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 588,
            title: "Plinten".to_string(),
            price: 4.5,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 589,
            title: "Stucwerk".to_string(),
            price: 5.71,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 582,
            title: "Verwijderen van kit (silicone)".to_string(),
            price: 5.0,
            unit: "".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 591,
            title: "Dilatatie <15mm".to_string(),
            price: 15.0,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 592,
            title: "Dilatatie >15 <25mm".to_string(),
            price: 16.09,
            unit: "per meter".to_string(),
            is_extra: false,
        },
    ];

    let active_services = if services.is_empty() {
        &default_services
    } else {
        services
    };

    let mut options_html = String::from("<option value=\"\">Kies een klus</option>\n");
    for svc in active_services.iter().filter(|s| !s.is_extra) {
        options_html.push_str(&format!(
            "<option value=\"{}\" data-price=\"{}\" data-unit=\"{}\">{}</option>\n",
            svc.id,
            svc.price,
            escape_html(&svc.unit),
            escape_html(&svc.title)
        ));
    }

    format!(
        r#"<div class="spb-kw-calculator et_pb_module" data-start-fee="150">
  <div class="spb-kw-success" style="display:none"></div>
  <div class="spb-kw-step spb-kw-step-calculator">
    <h3 class="et_pb_module_header">Kitwerk prijsindicatie</h3>
    <div class="spb-kw-copy spb-kw-copy--step1">
      <p>Bereken snel jouw prijsindicatie voor professioneel kitwerk en stuur deze naar je inbox.</p>
      <ul class="spb-kw-icons">
        <li>🔒 Wij gaan veilig met jouw gegevens om.</li>
        <li>⚡ Supersnel een indicatie in jouw mailbox.</li>
        <li>🛠️ Strak, duurzaam kitwerk door vakmensen.</li>
      </ul>
    </div>
    <div class="spb-kw-top-controls">
      <select class="spb-kw-service-select" style="display:none">
        {options_html}
      </select>
      <div class="spb-kw-step1-services"></div>
    </div>
    <div class="spb-kw-actions">
      <button type="button" class="et_pb_button spb-kw-next">Volgende</button>
    </div>
  </div>
  <div class="spb-kw-step spb-kw-step-lead" style="display:none">
    <form class="spb-kw-lead-form" novalidate>
      <h4 class="spb-kw-lead-title">Ontvang jouw kitwerk prijs per e-mail</h4>
      <div class="spb-kw-copy spb-kw-copy--step2">
        <p>🧾 Overzicht van jouw selectie:</p>
      </div>
      <div class="spb-kw-review" aria-live="polite"></div>
      <p class="spb-kw-info-line">🚗 Reisvergoeding: <span class="spb-kw-travel-rate">€ 0,73</span> per km (niet meegerekend in de prijs)</p>
      <div class="spb-kw-copy spb-kw-copy--step2">
        <p>🔒 We gebruiken je gegevens alleen voor jouw kitwerk-aanvraag.</p>
        <p>✉️ Geen spam. Je ontvangt één bevestiging met jouw prijsindicatie.</p>
      </div>
      <input type="text" name="name" placeholder="Naam" required autocomplete="name" />
      <input type="email" name="email" placeholder="E-mail" required autocomplete="email" />
      <input type="tel" name="phone" placeholder="Telefoon" autocomplete="tel" />
      <input type="text" name="city" placeholder="Woonplaats" autocomplete="address-level2" />
      <label class="spb-kw-consent">
        <input type="checkbox" name="consent" required />
        <span>Ik ga akkoord met de <a href="/privacy-policy" target="_blank" rel="noopener">privacy voorwaarden</a>.</span>
      </label>
      <input type="hidden" name="selected" />
      <input type="hidden" name="total" />
      <div class="spb-kw-lead-actions">
        <button type="button" class="et_pb_button spb-kw-prev">Vorige</button>
        <button type="submit" class="et_pb_button">Verstuur</button>
      </div>
      <div class="spb-kw-message" aria-live="polite"></div>
    </form>
  </div>
</div>
<script>
(function(){{
  function fmtEUR(v){{try{{return new Intl.NumberFormat('nl-NL',{{style:'currency',currency:'EUR'}}).format(v);}}catch(e){{return '€ '+(v||0).toFixed(2);}}}}
  document.querySelectorAll('.spb-kw-calculator').forEach(function(root){{
    if(root.dataset.npBound==='1')return;
    root.dataset.npBound='1';
    var startFee=parseFloat(root.dataset.startFee||'150')||150;
    var srcSelect=root.querySelector('.spb-kw-service-select');
    var step1=root.querySelector('.spb-kw-step1-services');
    if(!srcSelect||!step1)return;
    step1.innerHTML='<div class="spb-kw-step1-list"><div class="spb-kw-rows"></div></div><div class="spb-kw-actions" style="margin-top:10px"><button type="button" class="spb-kw-add-service">Voeg nog een dienst toe</button></div>';
    var rowsEl=step1.querySelector('.spb-kw-rows');
    function addRow(){{
      var r=document.createElement('div');
      r.className='spb-kw-row';
      r.innerHTML='<div class="spb-kw-row-field spb-kw-row-meters-wrap"><label>Meters</label><input type="number" min="0" max="999" step="1" class="spb-kw-row-meters" placeholder="0" inputmode="numeric" /></div><div class="spb-kw-row-field spb-kw-row-service-wrap"><label>Dienst</label><select class="spb-kw-row-service">'+srcSelect.innerHTML+'</select></div><div class="spb-kw-row-actions"><button type="button" class="spb-kw-remove-row" title="Verwijder">×</button></div>';
      rowsEl.appendChild(r);
      var all=rowsEl.querySelectorAll('.spb-kw-row');
      all.forEach(function(el,idx){{var btn=el.querySelector('.spb-kw-remove-row');if(btn)btn.style.display=idx>0?'inline-block':'none';}});
      r.querySelector('.spb-kw-remove-row').addEventListener('click',function(){{r.remove();}});
    }}
    addRow();
    step1.querySelector('.spb-kw-add-service').addEventListener('click',addRow);
    var nextBtn=root.querySelector('.spb-kw-next');
    var prevBtn=root.querySelector('.spb-kw-prev');
    var stepCalc=root.querySelector('.spb-kw-step-calculator');
    var stepLead=root.querySelector('.spb-kw-step-lead');
    if(nextBtn){{
      nextBtn.addEventListener('click',function(e){{
        e.preventDefault();
        stepCalc.style.display='none';
        var review=root.querySelector('.spb-kw-review');
        var tbodyHtml='';
        var baseSum=0;
        var selected=[];
        rowsEl.querySelectorAll('.spb-kw-row').forEach(function(r){{
          var sel=r.querySelector('select');
          var inp=r.querySelector('input[type="number"]');
          var id=parseInt(sel&&sel.value,10)||0;
          var meters=parseFloat(inp&&inp.value)||0;
          var opt=sel&&sel.options[sel.selectedIndex];
          var price=parseFloat(opt&&opt.getAttribute('data-price'))||0;
          var unit=(opt&&opt.getAttribute('data-unit'))||'';
          if(id&&meters>0&&price>0){{
            var line=price*meters;
            baseSum+=line;
            selected.push({{id:id,qty:meters,service:opt.text}});
            tbodyHtml+='<tr><td class="col-item">'+opt.text+(unit?' <small>/ '+unit+'</small>':'')+'</td><td class="col-qty">'+meters+'</td><td class="col-price">'+fmtEUR(price)+'</td><td class="col-total">'+fmtEUR(line)+'</td></tr>';
          }}
        }});
        var total=Math.max(startFee,baseSum);
        if(startFee>0&&total>baseSum){{
          tbodyHtml+='<tr><td class="col-item">Start tarief (minimum)</td><td class="col-qty">—</td><td class="col-price">'+fmtEUR(startFee)+'</td><td class="col-total">'+fmtEUR(startFee)+'</td></tr>';
        }}
        review.innerHTML='<table class="spb-kw-review-table"><thead><tr><th class="col-item">Item</th><th class="col-qty">Aantal</th><th class="col-price">Prijs</th><th class="col-total">Totaal</th></tr></thead><tbody class="spb-kw-rows">'+tbodyHtml+'</tbody><tfoot><tr><th colspan="3" style="text-align:right">Indicatie (incl. btw)</th><th class="spb-kw-grand-total">'+fmtEUR(total)+'</th></tr></tfoot></table>';
        root.querySelector('input[name="selected"]').value=JSON.stringify(selected);
        root.querySelector('input[name="total"]').value=total.toFixed(2);
        stepLead.style.display='block';
      }});
    }}
    if(prevBtn){{
      prevBtn.addEventListener('click',function(e){{
        e.preventDefault();
        stepLead.style.display='none';
        stepCalc.style.display='block';
      }});
    }}
    var form=root.querySelector('.spb-kw-lead-form');
    if(form){{
      form.addEventListener('submit',function(e){{
        e.preventDefault();
        var msgEl=form.querySelector('.spb-kw-message');
        var consent=form.querySelector('input[name="consent"]');
        if(!consent||!consent.checked){{
          if(msgEl)msgEl.textContent='Vink de privacy voorwaarden aan.';
          return;
        }}
        var payload={{
          name:form.querySelector('input[name="name"]').value,
          email:form.querySelector('input[name="email"]').value,
          phone:form.querySelector('input[name="phone"]').value,
          city:form.querySelector('input[name="city"]').value,
          selected:form.querySelector('input[name="selected"]').value,
          total:form.querySelector('input[name="total"]').value
        }};
        fetch('/wp-json/neopress/v1/forms/1/submit',{{
          method:'POST',
          headers:{{'Content-Type':'application/json'}},
          body:JSON.stringify(payload)
        }}).then(function(r){{
          if(r.ok){{
            stepCalc.style.display='none';
            stepLead.style.display='none';
            var succ=root.querySelector('.spb-kw-success');
            if(succ){{succ.textContent='Bedankt! Je ontvangt snel een mail met de prijsindicatie. Neem direct contact op voor vragen.';succ.style.display='block';}}
          }}else if(msgEl){{
            msgEl.textContent='Er ging iets mis.';
          }}
        }});
      }});
    }}
  }});
}})();
</script>"#,
        options_html = options_html
    )
}
