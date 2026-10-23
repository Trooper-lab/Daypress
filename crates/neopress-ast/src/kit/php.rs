use serde_json::{Map, Value};

/// Zero-panic recursive parser for PHP `serialize()` payloads (`a:...`, `s:...`, `i:...`, `d:...`, `b:...`, `N;`),
/// converting them into `serde_json::Value`.
pub fn parse_php_serialized_to_json(raw: &str) -> Option<Value> {
    let bytes = raw.trim().as_bytes();
    let mut pos = 0;
    parse_php_val(bytes, &mut pos)
}

fn parse_php_val(bytes: &[u8], pos: &mut usize) -> Option<Value> {
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    if *pos >= bytes.len() {
        return None;
    }
    let tag = bytes[*pos];
    *pos += 1;
    match tag {
        b'N' => {
            if *pos < bytes.len() && bytes[*pos] == b';' {
                *pos += 1;
                Some(Value::Null)
            } else {
                None
            }
        }
        b'b' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            Some(Value::Bool(val_str.trim() == "1"))
        }
        b'i' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            let n = val_str.trim().parse::<i64>().ok()?;
            Some(Value::Number(n.into()))
        }
        b'd' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            let f = val_str.trim().parse::<f64>().ok()?;
            serde_json::Number::from_f64(f).map(Value::Number)
        }
        b's' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let len_start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b':' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let len_str = std::str::from_utf8(&bytes[len_start..*pos]).ok()?;
            let str_len = len_str.trim().parse::<usize>().ok()?;
            *pos += 1; // skip ':'
            if *pos >= bytes.len() || bytes[*pos] != b'"' {
                return None;
            }
            *pos += 1; // skip opening '"'
            if *pos + str_len > bytes.len() {
                return None;
            }
            let s = String::from_utf8_lossy(&bytes[*pos..*pos + str_len]).to_string();
            *pos += str_len;
            if *pos < bytes.len() && bytes[*pos] == b'"' {
                *pos += 1;
            }
            if *pos < bytes.len() && bytes[*pos] == b';' {
                *pos += 1;
            }
            Some(Value::String(s))
        }
        b'a' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let count_start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b':' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let count_str = std::str::from_utf8(&bytes[count_start..*pos]).ok()?;
            let count = count_str.trim().parse::<usize>().ok()?;
            *pos += 1; // skip ':'
            if *pos >= bytes.len() || bytes[*pos] != b'{' {
                return None;
            }
            *pos += 1; // skip '{'

            let mut entries: Vec<(Value, Value)> = Vec::with_capacity(count);
            for _ in 0..count {
                let k = parse_php_val(bytes, pos)?;
                let v = parse_php_val(bytes, pos)?;
                entries.push((k, v));
            }
            while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
                *pos += 1;
            }
            if *pos < bytes.len() && bytes[*pos] == b'}' {
                *pos += 1;
            }

            let is_sequential_list = entries.iter().enumerate().all(|(idx, (k, _))| {
                k.as_i64() == Some(idx as i64)
            });

            if is_sequential_list {
                Some(Value::Array(entries.into_iter().map(|(_, v)| v).collect()))
            } else {
                let mut map = Map::new();
                for (k, v) in entries {
                    let key_str = match k {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        _ => continue,
                    };
                    map.insert(key_str, v);
                }
                Some(Value::Object(map))
            }
        }
        _ => None,
    }
}
