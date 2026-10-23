use regex::Regex;

/// Transliterates Latin-1 Supplement and Latin Extended-A/B accented characters
/// into their ASCII equivalents, matching WordPress `remove_accents()` and
/// PHP `iconv('utf-8', 'us-ascii//TRANSLIT')`.
pub fn remove_accents(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            // Latin-1 Supplement Uppercase
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => out.push('A'),
            'Æ' => out.push_str("AE"),
            'Ç' => out.push('C'),
            'È' | 'É' | 'Ê' | 'Ë' => out.push('E'),
            'Ì' | 'Í' | 'Î' | 'Ï' => out.push('I'),
            'Ð' => out.push('D'),
            'Ñ' => out.push('N'),
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => out.push('O'),
            'Ù' | 'Ú' | 'Û' | 'Ü' => out.push('U'),
            'Ý' => out.push('Y'),
            'Þ' => out.push_str("TH"),
            'ß' => out.push_str("ss"),

            // Latin-1 Supplement Lowercase
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => out.push('a'),
            'æ' => out.push_str("ae"),
            'ç' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ð' => out.push('d'),
            'ñ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => out.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'þ' => out.push_str("th"),

            // Latin Extended-A & Extended-B
            'Ā' | 'Ă' | 'Ą' => out.push('A'),
            'ā' | 'ă' | 'ą' => out.push('a'),
            'Ć' | 'Ĉ' | 'Ċ' | 'Č' => out.push('C'),
            'ć' | 'ĉ' | 'ċ' | 'č' => out.push('c'),
            'Ď' | 'Đ' => out.push('D'),
            'ď' | 'đ' => out.push('d'),
            'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => out.push('E'),
            'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => out.push('e'),
            'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => out.push('G'),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' => out.push('g'),
            'Ĥ' | 'Ħ' => out.push('H'),
            'ĥ' | 'ħ' => out.push('h'),
            'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => out.push('I'),
            'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => out.push('i'),
            'Ĳ' => out.push_str("IJ"),
            'ĳ' => out.push_str("ij"),
            'Ĵ' => out.push('J'),
            'ĵ' => out.push('j'),
            'Ķ' => out.push('K'),
            'ķ' | 'ĸ' => out.push('k'),
            'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => out.push('L'),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => out.push('l'),
            'Ń' | 'Ņ' | 'Ň' | 'Ŋ' => out.push('N'),
            'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => out.push('n'),
            'Ō' | 'Ŏ' | 'Ő' => out.push('O'),
            'ō' | 'ŏ' | 'ő' => out.push('o'),
            'Œ' => out.push_str("OE"),
            'œ' => out.push_str("oe"),
            'Ŕ' | 'Ŗ' | 'Ř' => out.push('R'),
            'ŕ' | 'ŗ' | 'ř' => out.push('r'),
            'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'Ș' => out.push('S'),
            'ś' | 'ŝ' | 'ş' | 'š' | 'ș' | 'ſ' => out.push('s'),
            'Ţ' | 'Ť' | 'Ŧ' | 'Ț' => out.push('T'),
            'ţ' | 'ť' | 'ŧ' | 'ț' => out.push('t'),
            'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => out.push('U'),
            'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => out.push('u'),
            'Ŵ' => out.push('W'),
            'ŵ' => out.push('w'),
            'Ŷ' | 'Ÿ' => out.push('Y'),
            'ŷ' => out.push('y'),
            'Ź' | 'Ż' | 'Ž' => out.push('Z'),
            'ź' | 'ż' | 'ž' => out.push('z'),

            // Strip combining diacritical marks
            '\u{0300}'..='\u{036F}' => {}

            other => out.push(other),
        }
    }
    out
}

/// WordPress `sanitize_title_with_dashes()` port:
/// Strips HTML tags, transliterates accents via `remove_accents()`, replaces
/// non-alphanumeric sequences with `-`, trims leading/trailing `-`, and lowercases.
/// Preserves non-ASCII Unicode alphanumeric characters (Cyrillic, Greek, Japanese, Arabic, Hebrew).
pub fn sanitize_title_with_dashes(title: &str) -> String {
    let re_tags = Regex::new(r"<[^>]*>").expect("valid html tag regex");
    let stripped = re_tags.replace_all(title, "");
    let unaccented = remove_accents(&stripped);

    let mut out = String::with_capacity(unaccented.len());
    let mut prev_is_dash = true;

    for ch in unaccented.chars() {
        if ch.is_alphanumeric() {
            for lower_ch in ch.to_lowercase() {
                out.push(lower_ch);
            }
            prev_is_dash = false;
        } else if !prev_is_dash {
            out.push('-');
            prev_is_dash = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }

    out.chars().take(200).collect()
}

/// Transpiled & Golden-Master verified `slugify()` matching `fixtures/slugify.php`.
/// Retains Unicode alphanumeric characters, avoiding `"n-a"` collapse for non-Latin titles.
pub fn slugify(text: &str) -> String {
    let sanitized = sanitize_title_with_dashes(text);
    if sanitized.is_empty() {
        "n-a".to_string()
    } else {
        sanitized
    }
}
