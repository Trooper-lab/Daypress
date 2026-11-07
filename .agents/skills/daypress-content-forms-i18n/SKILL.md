---
name: daypress-content-forms-i18n
description: >-
  Model Custom Post Types (CPTs), ACF-parity Structured Field Groups, filterable
  LoopGrid collections, FormStudio forms, multilingual (i18n) content, secret
  stakeholder preview links, and scheduled revisions in Daypress. Use this skill
  when building dynamic content directories, contact/lead-gen forms, or
  multi-language websites for a user.
---

# Daypress Content Modeling, FormStudio & Multilingual (`i18n`) Skill

Use this skill whenever a user asks you to **create Custom Post Types (portfolios, case studies, team members, job boards, docs, events), add custom ACF-style structured fields, build contact/lead-generation forms, or translate their site into multiple languages**.

---

## 1. Custom Post Types & 1-Click Starter Blueprints

### A. 1-Click Blueprint Provisioning (`daypress_provision_cpt_preset`)
Daypress includes 7 complete blueprints that automatically create the Custom Post Type, custom taxonomy, ACF Structured Field Group, and Single/Archive templates in one call:
- `"case_studies"` (`case_study`)
- `"portfolio"` (`portfolio`)
- `"team"` (`team_member`)
- `"docs"` (`doc`)
- `"jobs"` (`job_listing`)
- `"events"` (`event`)
- `"testimonials"` (`testimonial`)

```json
{
  "name": "daypress_provision_cpt_preset",
  "arguments": {
    "preset": "case_studies"
  }
}
```

### B. Custom Post Type Definitions (`daypress_save_post_type`)
To register a bespoke Custom Post Type:
```json
{
  "name": "daypress_save_post_type",
  "arguments": {
    "slug": "product_guide",
    "singular_label": "Product Guide",
    "plural_label": "Product Guides",
    "description": "In-depth technical guides and architecture notes.",
    "menu_icon": "fa-solid fa-book-open",
    "public": true,
    "has_archive": true,
    "hierarchical": false,
    "show_in_rest": true,
    "rewrite_slug": "guides",
    "supports": ["title", "editor", "thumbnail", "excerpt", "custom-fields"],
    "taxonomies": ["category", "post_tag"]
  }
}
```
Once registered, create items using `daypress_create_post` with `"post_type": "product_guide"`, and display them on any page with a `LoopGrid` AST node (`"post_type": "product_guide"`).

---

## 2. ACF-Parity Structured Field Groups (`daypress_save_field_group`)

Attach strongly-typed custom fields to any `post_type` (`post`, `page`, or custom CPT):

### Supported `field_type` Values
`text`, `textarea`, `wysiwyg`, `number`, `date`, `url`, `true_false`, `select`, `checkbox`, `media`, `post_object`, `repeater`.

```json
{
  "name": "daypress_save_field_group",
  "arguments": {
    "group_key": "group_case_study_metrics",
    "title": "Case Study Details",
    "description": "Client metadata and headline metrics displayed on cards and single views.",
    "position": "normal",
    "is_active": true,
    "location_rules": [
      { "param": "post_type", "operator": "==", "value": "case_study" }
    ],
    "fields": [
      {
        "key": "field_client_name",
        "name": "client_name",
        "label": "Client Name",
        "field_type": "text",
        "required": true,
        "show_in_collection_card": true
      },
      {
        "key": "field_latency_gain",
        "name": "latency_gain",
        "label": "Latency Improvement",
        "field_type": "text",
        "placeholder": "e.g. 12x faster SSR",
        "show_in_collection_card": true
      },
      {
        "key": "field_live_url",
        "name": "live_url",
        "label": "Live Project URL",
        "field_type": "url"
      }
    ]
  }
}
```

### Rendering Structured Fields on the Frontend
1. **In Dynamic Theme Templates (`daypress_save_template`)**: Use `{{acf.<field_name>}}` tokens (e.g. `{{acf.client_name}}`, `{{acf.latency_gain}}`) inside `Heading`, `Text`, `Button`, or `Image` nodes.
2. **In Filterable Collection Grids (`LoopGrid`)**: Pass `"meta_fields": ["client_name", "latency_gain"]` in the `LoopGrid` node's `data` object (or set `"show_in_collection_card": true` on the field definition) to render structured field badges on every card.

---

## 3. Building Contact, Lead-Gen & Multi-Step Forms (`FormStudio`)

Daypress includes **FormStudio** (`/d-admin/forms`) with a SQLite-first Submissions Vault, honeypot + time-lock + rate-limit anti-spam, conditional field visibility, calculated formulas, and SMTP notifications.

### Step 1: Create the Form (`daypress_create_form`)
Supported `field_type` values: `text`, `email`, `tel`, `url`, `number`, `date`, `textarea`, `select`, `radio`, `checkbox`, `acceptance`, `file`, `rating`, `range`, `calculation`, `hidden`.
Supported `width` values: `"full"`, `"half"` (`1/2`), `"third"` (`1/3`), `"two_thirds"` (`2/3`).

```json
{
  "name": "daypress_create_form",
  "arguments": {
    "title": "Project Inquiry",
    "fields": [
      {
        "name": "full_name",
        "label": "Your Name",
        "field_type": "text",
        "required": true,
        "placeholder": "Ada Lovelace",
        "width": "half",
        "step": 1
      },
      {
        "name": "email",
        "label": "Work Email",
        "field_type": "email",
        "required": true,
        "placeholder": "ada@example.com",
        "width": "half",
        "step": 1
      },
      {
        "name": "project_scope",
        "label": "Project Type",
        "field_type": "select",
        "required": true,
        "options": ["New Daypress Website", "WordPress Migration", "Custom Design System"],
        "width": "full",
        "step": 1
      },
      {
        "name": "message",
        "label": "Tell us about your project",
        "field_type": "textarea",
        "required": true,
        "placeholder": "Timeline, goals, and current site URL...",
        "width": "full",
        "step": 1
      }
    ],
    "settings": {
      "submit_label": "Send Inquiry →",
      "success_message": "Thank you! We have received your inquiry and will reply within 1 business day.",
      "recipient_email": "hello@example.com",
      "enable_honeypot": true,
      "rate_limit_per_minute": 10
    }
  }
}
```

### Step 2: Embed the Form on Any Page
- **In DayStudio AST (`daypress_update_post_layout`)**: Add a `Form` node:
  ```json
  {
    "id": "contact_form_block",
    "type": "Form",
    "data": {
      "form_id": 1,
      "title": "Start a Conversation",
      "button_text": "Send Inquiry →"
    },
    "styles": {}
  }
  ```
- **In Post/Page HTML (`daypress_create_post` / `daypress_update_post`)**: Include the shortcode `[daypress_form id="1"]`.

---

## 4. Multilingual (`i18n`) Websites & Editorial Workflows

### A. Translating Posts & Pages (`daypress_list_languages` & `daypress_link_translations`)
Daypress supports Polylang/WPML-compatible multi-language routing (`/slug/` for default language, `/de/slug/`, `/nl/slug/`, `/fr/slug/`, `/es/slug/` for secondary locales) with automatic `<link rel="alternate" hreflang="...">` tags and `sitemap.xml` `xhtml:link` alternates.

1. **Duplicate the source page** using `daypress_duplicate_post`:
   ```json
   { "name": "daypress_duplicate_post", "arguments": { "post_id": 3 } }
   ```
2. **Translate the cloned page's title, slug, and layout AST**, and publish it via `daypress_update_post` and `daypress_update_post_layout`.
3. **Link the translations** via `daypress_link_translations` (auto-enables the target language in `daypress_languages` when `upsert_language_code` is provided):
   ```json
   {
     "name": "daypress_link_translations",
     "arguments": {
       "source_post_id": 3,
       "target_post_id": 14,
       "target_lang": "nl",
       "upsert_language_code": "nl"
     }
   }
   ```
4. **Add a Language Switcher** to the site header using the `LanguageSwitcher` AST node (`{"type": "LanguageSwitcher", "data": {"display_style": "pills", "show_flags": true, "show_names": false, "show_codes": true, "current_lang": "en", "links": []}}`) or shortcode `[daypress_language_switcher]`.

### B. Editorial Previews, Scheduled Revisions & Visual AST Diffs
- **Secret Draft Preview Links (`daypress_create_preview_link`)**: Generate a login-free `/preview/{token}` link so the user or their client can review an unpublished draft.
- **Scheduled Revisions (`daypress_schedule_revision`)**: Schedule a future content/layout swap on an already-published page without taking the live URL offline.
- **Structural AST Diff (`daypress_diff_revisions`)**: Compare any two revisions (or compare a revision against the live page) node-by-node.
