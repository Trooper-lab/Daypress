//! Built-in `/d-admin/` UI Localization Catalogs (`en`, `de`, `nl`, `fr`, `es`)
//! and ISO Language Metadata Registry (`crates/daypress-server/src/i18n.rs`).

/// Normalizes a WordPress locale (`"de_DE"`, `"nl_NL"`, `"fr_FR"`, `"es_ES"`, `"en_US"`)
/// or 2-letter ISO 639-1 code (`"de"`, `"nl"`, `"fr"`, `"es"`, `"en"`) into one of the
/// 5 built-in admin UI dictionary codes.
pub fn normalize_ui_lang(raw: &str) -> &'static str {
    let clean = raw.trim().to_ascii_lowercase();
    let prefix = clean.split(['_', '-']).next().unwrap_or("en");
    match prefix {
        "de" => "de",
        "nl" => "nl",
        "fr" => "fr",
        "es" => "es",
        _ => "en",
    }
}

/// Standard preset languages: `(code, name, native_name, flag_emoji, is_rtl)`.
pub const PRESET_LANGUAGES: &[(&str, &str, &str, &str, bool)] = &[
    ("en", "English", "English", "🇺🇸", false),
    ("de", "German", "Deutsch", "🇩🇪", false),
    ("nl", "Dutch", "Nederlands", "🇳🇱", false),
    ("fr", "French", "Français", "🇫🇷", false),
    ("es", "Spanish", "Español", "🇪🇸", false),
    ("it", "Italian", "Italiano", "🇮🇹", false),
    ("pt", "Portuguese", "Português", "🇵🇹", false),
    ("pl", "Polish", "Polski", "🇵🇱", false),
    ("sv", "Swedish", "Svenska", "🇸🇪", false),
    ("da", "Danish", "Dansk", "🇩🇰", false),
    ("ja", "Japanese", "日本語", "🇯🇵", false),
    ("zh", "Chinese", "中文", "🇨🇳", false),
    ("ar", "Arabic", "العربية", "🇸🇦", true),
    ("he", "Hebrew", "עברית", "🇮🇱", true),
];

/// Returns `(locale, display_name, flag_emoji, is_rtl)` for standard ISO 639-1 codes.
pub fn preset_language_info(code: &str) -> (&'static str, &'static str, &'static str, bool) {
    match code.trim().to_ascii_lowercase().as_str() {
        "en" | "en_us" | "en-us" => ("en_US", "English", "🇺🇸", false),
        "en_gb" | "en-gb" => ("en_GB", "English (UK)", "🇬🇧", false),
        "de" | "de_de" | "de-de" => ("de_DE", "Deutsch", "🇩🇪", false),
        "nl" | "nl_nl" | "nl-nl" => ("nl_NL", "Nederlands", "🇳🇱", false),
        "fr" | "fr_fr" | "fr-fr" => ("fr_FR", "Français", "🇫🇷", false),
        "es" | "es_es" | "es-es" => ("es_ES", "Español", "🇪🇸", false),
        "it" | "it_it" | "it-it" => ("it_IT", "Italiano", "🇮🇹", false),
        "pt" | "pt_pt" | "pt_br" => ("pt_PT", "Português", "🇵🇹", false),
        "pl" | "pl_pl" => ("pl_PL", "Polski", "🇵🇱", false),
        "sv" | "sv_se" => ("sv_SE", "Svenska", "🇸🇪", false),
        "da" | "da_dk" => ("da_DK", "Dansk", "🇩🇰", false),
        "ja" => ("ja", "日本語", "🇯🇵", false),
        "zh" | "zh_cn" => ("zh_CN", "中文", "🇨🇳", false),
        "ar" => ("ar", "العربية", "🇸🇦", true),
        "he" | "he_il" => ("he_IL", "עברית", "🇮🇱", true),
        _ => ("en_US", "Language", "🌐", false),
    }
}

/// Zero-allocation dictionary lookup for `/d-admin/` UI strings across `en`, `de`, `nl`, `fr`, and `es`.
/// Falls back to `key` if an unknown key is passed.
pub fn t<'a>(locale_or_code: &str, key: &'a str) -> &'a str {
    let lang = normalize_ui_lang(locale_or_code);
    match (lang, key) {
        // Navigation Sidebar & Core Sections
        ("en", "Dashboard") | ("en", "nav.dashboard") => "Dashboard",
        ("de", "Dashboard") | ("de", "nav.dashboard") => "Dashboard",
        ("nl", "Dashboard") | ("nl", "nav.dashboard") => "Dashboard",
        ("fr", "Dashboard") | ("fr", "nav.dashboard") => "Tableau de bord",
        ("es", "Dashboard") | ("es", "nav.dashboard") => "Escritorio",

        ("en", "Home") | ("en", "nav.home") => "Home",
        ("de", "Home") | ("de", "nav.home") => "Startseite",
        ("nl", "Home") | ("nl", "nav.home") => "Startpagina",
        ("fr", "Home") | ("fr", "nav.home") => "Accueil",
        ("es", "Home") | ("es", "nav.home") => "Inicio",

        ("en", "nav.get_started") => "Get Started & AI",
        ("de", "nav.get_started") => "Erste Schritte & KI",
        ("nl", "nav.get_started") => "Aan de slag & AI",
        ("fr", "nav.get_started") => "Premiers pas & IA",
        ("es", "nav.get_started") => "Primeros pasos e IA",

        ("en", "Analytics") | ("en", "nav.analytics") => "Analytics",
        ("de", "Analytics") | ("de", "nav.analytics") => "Analysen",
        ("nl", "Analytics") | ("nl", "nav.analytics") => "Statistieken",
        ("fr", "Analytics") | ("fr", "nav.analytics") => "Analytique",
        ("es", "Analytics") | ("es", "nav.analytics") => "Analítica",

        ("en", "Updates") | ("en", "nav.updates") => "Updates",
        ("de", "Updates") | ("de", "nav.updates") => "Aktualisierungen",
        ("nl", "Updates") | ("nl", "nav.updates") => "Updates",
        ("fr", "Updates") | ("fr", "nav.updates") => "Mises à jour",
        ("es", "Updates") | ("es", "nav.updates") => "Actualizaciones",

        ("en", "Posts") | ("en", "nav.posts") => "Posts",
        ("de", "Posts") | ("de", "nav.posts") => "Beiträge",
        ("nl", "Posts") | ("nl", "nav.posts") => "Berichten",
        ("fr", "Posts") | ("fr", "nav.posts") => "Articles",
        ("es", "Posts") | ("es", "nav.posts") => "Entradas",

        ("en", "All Posts") | ("en", "nav.all_posts") => "All Posts",
        ("de", "All Posts") | ("de", "nav.all_posts") => "Alle Beiträge",
        ("nl", "All Posts") | ("nl", "nav.all_posts") => "Alle berichten",
        ("fr", "All Posts") | ("fr", "nav.all_posts") => "Tous les articles",
        ("es", "All Posts") | ("es", "nav.all_posts") => "Todas las entradas",

        ("en", "Add New") | ("en", "nav.add_new") => "Add New",
        ("de", "Add New") | ("de", "nav.add_new") => "Erstellen",
        ("nl", "Add New") | ("nl", "nav.add_new") => "Nieuwe toevoegen",
        ("fr", "Add New") | ("fr", "nav.add_new") => "Ajouter",
        ("es", "Add New") | ("es", "nav.add_new") => "Añadir nuevo",

        ("en", "Categories") | ("en", "nav.categories") => "Categories",
        ("de", "Categories") | ("de", "nav.categories") => "Kategorien",
        ("nl", "Categories") | ("nl", "nav.categories") => "Categorieën",
        ("fr", "Categories") | ("fr", "nav.categories") => "Catégories",
        ("es", "Categories") | ("es", "nav.categories") => "Categorías",

        ("en", "Tags") | ("en", "nav.tags") => "Tags",
        ("de", "Tags") | ("de", "nav.tags") => "Schlagwörter",
        ("nl", "Tags") | ("nl", "nav.tags") => "Tags",
        ("fr", "Tags") | ("fr", "nav.tags") => "Étiquettes",
        ("es", "Tags") | ("es", "nav.tags") => "Etiquetas",

        ("en", "nav.media") => "Media",
        ("de", "nav.media") => "Medien",
        ("nl", "nav.media") => "Media",
        ("fr", "nav.media") => "Médias",
        ("es", "nav.media") => "Medios",

        ("en", "nav.library") => "Library",
        ("de", "nav.library") => "Mediathek",
        ("nl", "nav.library") => "Bibliotheek",
        ("fr", "nav.library") => "Médiathèque",
        ("es", "nav.library") => "Biblioteca",

        ("en", "nav.pages") => "Pages",
        ("de", "nav.pages") => "Seiten",
        ("nl", "nav.pages") => "Pagina's",
        ("fr", "nav.pages") => "Pages",
        ("es", "nav.pages") => "Páginas",

        ("en", "nav.all_pages") => "All Pages",
        ("de", "nav.all_pages") => "Alle Seiten",
        ("nl", "nav.all_pages") => "Alle pagina's",
        ("fr", "nav.all_pages") => "Toutes les pages",
        ("es", "nav.all_pages") => "Todas las páginas",

        ("en", "nav.content_model") => "Content Model",
        ("de", "nav.content_model") => "Inhaltsmodell",
        ("nl", "nav.content_model") => "Contentmodel",
        ("fr", "nav.content_model") => "Modèle de contenu",
        ("es", "nav.content_model") => "Modelo de contenido",

        ("en", "nav.daystudio") => "DayStudio",
        ("de", "nav.daystudio") => "DayStudio",
        ("nl", "nav.daystudio") => "DayStudio",
        ("fr", "nav.daystudio") => "DayStudio",
        ("es", "nav.daystudio") => "DayStudio",

        ("en", "nav.comments") => "Comments",
        ("de", "nav.comments") => "Kommentare",
        ("nl", "nav.comments") => "Reacties",
        ("fr", "nav.comments") => "Commentaires",
        ("es", "nav.comments") => "Comentarios",

        ("en", "nav.appearance") => "Appearance",
        ("de", "nav.appearance") => "Design",
        ("nl", "nav.appearance") => "Weergave",
        ("fr", "nav.appearance") => "Apparence",
        ("es", "nav.appearance") => "Apariencia",

        ("en", "nav.plugins") => "Plugins",
        ("de", "nav.plugins") => "Plugins",
        ("nl", "nav.plugins") => "Plugins",
        ("fr", "nav.plugins") => "Extensions",
        ("es", "nav.plugins") => "Plugins",

        ("en", "nav.forms") => "Forms Studio",
        ("de", "nav.forms") => "Formular-Studio",
        ("nl", "nav.forms") => "Formulieren",
        ("fr", "nav.forms") => "Formulaires",
        ("es", "nav.forms") => "Formularios",

        ("en", "nav.users") => "Users",
        ("de", "nav.users") => "Benutzer",
        ("nl", "nav.users") => "Gebruikers",
        ("fr", "nav.users") => "Comptes",
        ("es", "nav.users") => "Usuarios",

        ("en", "nav.tools") => "Tools",
        ("de", "nav.tools") => "Werkzeuge",
        ("nl", "nav.tools") => "Gereedschap",
        ("fr", "nav.tools") => "Outils",
        ("es", "nav.tools") => "Herramientas",

        ("en", "nav.settings") => "Settings",
        ("de", "nav.settings") => "Einstellungen",
        ("nl", "nav.settings") => "Instellingen",
        ("fr", "nav.settings") => "Réglages",
        ("es", "nav.settings") => "Ajustes",

        ("en", "nav.languages") => "Languages",
        ("de", "nav.languages") => "Sprachen",
        ("nl", "nav.languages") => "Talen",
        ("fr", "nav.languages") => "Langues",
        ("es", "nav.languages") => "Idiomas",

        ("en", "nav.publish") => "Publish",
        ("de", "nav.publish") => "Veröffentlichen",
        ("nl", "nav.publish") => "Publiceren",
        ("fr", "nav.publish") => "Publier",
        ("es", "nav.publish") => "Publicar",

        ("en", "nav.engine") => "Engine Telemetry",
        ("de", "nav.engine") => "Engine-Telemetrie",
        ("nl", "nav.engine") => "Engine-telemetrie",
        ("fr", "nav.engine") => "Télémétrie",
        ("es", "nav.engine") => "Telemetría",

        ("en", "nav.logout") => "Log out",
        ("de", "nav.logout") => "Abmelden",
        ("nl", "nav.logout") => "Uitloggen",
        ("fr", "nav.logout") => "Déconnexion",
        ("es", "nav.logout") => "Cerrar sesión",

        ("en", "Media") => "Media",
        ("de", "Media") => "Medien",
        ("nl", "Media") => "Media",
        ("fr", "Media") => "Médias",
        ("es", "Media") => "Medios",

        ("en", "Pages") => "Pages",
        ("de", "Pages") => "Seiten",
        ("nl", "Pages") => "Pagina's",
        ("fr", "Pages") => "Pages",
        ("es", "Pages") => "Páginas",

        ("en", "All Pages") => "All Pages",
        ("de", "All Pages") => "Alle Seiten",
        ("nl", "All Pages") => "Alle pagina's",
        ("fr", "All Pages") => "Toutes les pages",
        ("es", "All Pages") => "Todas las páginas",

        ("en", "Comments") => "Comments",
        ("de", "Comments") => "Kommentare",
        ("nl", "Comments") => "Reacties",
        ("fr", "Comments") => "Commentaires",
        ("es", "Comments") => "Comentarios",

        ("en", "Content Model") => "Content Model",
        ("de", "Content Model") => "Inhaltsmodell",
        ("nl", "Content Model") => "Contentmodel",
        ("fr", "Content Model") => "Modèle de contenu",
        ("es", "Content Model") => "Modelo de contenido",

        ("en", "Languages") => "Languages",
        ("de", "Languages") => "Sprachen",
        ("nl", "Languages") => "Talen",
        ("fr", "Languages") => "Langues",
        ("es", "Languages") => "Idiomas",

        ("en", "Appearance") => "Appearance",
        ("de", "Appearance") => "Design",
        ("nl", "Appearance") => "Weergave",
        ("fr", "Appearance") => "Apparence",
        ("es", "Appearance") => "Apariencia",

        ("en", "Plugins") => "Plugins",
        ("de", "Plugins") => "Plugins",
        ("nl", "Plugins") => "Plugins",
        ("fr", "Plugins") => "Extensions",
        ("es", "Plugins") => "Plugins",

        ("en", "Users") => "Users",
        ("de", "Users") => "Benutzer",
        ("nl", "Users") => "Gebruikers",
        ("fr", "Users") => "Comptes",
        ("es", "Users") => "Usuarios",

        ("en", "Tools") => "Tools",
        ("de", "Tools") => "Werkzeuge",
        ("nl", "Tools") => "Gereedschap",
        ("fr", "Tools") => "Outils",
        ("es", "Tools") => "Herramientas",

        ("en", "Settings") => "Settings",
        ("de", "Settings") => "Einstellungen",
        ("nl", "Settings") => "Instellingen",
        ("fr", "Settings") => "Réglages",
        ("es", "Settings") => "Ajustes",

        // Common Editorial Statuses & Actions
        ("en", "Published") => "Published",
        ("de", "Published") => "Veröffentlicht",
        ("nl", "Published") => "Gepubliceerd",
        ("fr", "Published") => "Publié",
        ("es", "Published") => "Publicado",

        ("en", "Draft") => "Draft",
        ("de", "Draft") => "Entwurf",
        ("nl", "Draft") => "Concept",
        ("fr", "Draft") => "Brouillon",
        ("es", "Draft") => "Borrador",

        ("en", "Scheduled") => "Scheduled",
        ("de", "Scheduled") => "Geplant",
        ("nl", "Scheduled") => "Gepland",
        ("fr", "Scheduled") => "Planifié",
        ("es", "Scheduled") => "Programado",

        ("en", "Trash") => "Trash",
        ("de", "Trash") => "Papierkorb",
        ("nl", "Trash") => "Prullenbak",
        ("fr", "Trash") => "Corbeille",
        ("es", "Trash") => "Papelera",

        ("en", "Duplicate") => "Duplicate",
        ("de", "Duplicate") => "Duplizieren",
        ("nl", "Duplicate") => "Dupliceren",
        ("fr", "Duplicate") => "Dupliquer",
        ("es", "Duplicate") => "Duplicar",

        ("en", "Edit") => "Edit",
        ("de", "Edit") => "Bearbeiten",
        ("nl", "Edit") => "Bewerken",
        ("fr", "Edit") => "Modifier",
        ("es", "Edit") => "Editar",

        ("en", "View") => "View",
        ("de", "View") => "Ansehen",
        ("nl", "View") => "Bekijken",
        ("fr", "View") => "Voir",
        ("es", "View") => "Ver",

        ("en", "Save Changes") => "Save Changes",
        ("de", "Save Changes") => "Änderungen speichern",
        ("nl", "Save Changes") => "Wijzigingen opslaan",
        ("fr", "Save Changes") => "Enregistrer les modifications",
        ("es", "Save Changes") => "Guardar cambios",

        _ => key,
    }
}
