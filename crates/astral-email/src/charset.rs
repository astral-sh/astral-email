//! Python charset labels for the supported codecs.

/// Lowercase an email charset label, then apply CPython's codec-name normalization.
pub(crate) fn lookup(name: &str) -> Option<&'static str> {
    let common = match name.len() {
        4 if name.eq_ignore_ascii_case("utf8") => Some("utf-8"),
        5 if name.eq_ignore_ascii_case("utf-8") => Some("utf-8"),
        5 if name.eq_ignore_ascii_case("ascii") => Some("ascii"),
        6 if name.eq_ignore_ascii_case("latin1") => Some("iso8859-1"),
        6 if name.eq_ignore_ascii_case("cp1252") => Some("cp1252"),
        6 if name.eq_ignore_ascii_case("utf-16") => Some("utf-16"),
        7 if name.eq_ignore_ascii_case("latin-1") => Some("iso8859-1"),
        9 if name.eq_ignore_ascii_case("iso8859-1") => Some("iso8859-1"),
        9 if name.eq_ignore_ascii_case("utf-8-sig") => Some("utf-8-sig"),
        9 if name.eq_ignore_ascii_case("utf-16-be") => Some("utf-16-be"),
        9 if name.eq_ignore_ascii_case("utf-16-le") => Some("utf-16-le"),
        10 if name.eq_ignore_ascii_case("iso-8859-1") => Some("iso8859-1"),
        12 if name.eq_ignore_ascii_case("windows-1252") => Some("cp1252"),
        _ => None,
    };
    if common.is_some() {
        return common;
    }
    let mut normalized = String::with_capacity(name.len());
    let mut separator = false;
    for byte in name.to_lowercase().bytes() {
        if byte == 0 {
            return None;
        }
        if byte.is_ascii_alphanumeric() || byte == b'.' {
            if separator && !normalized.is_empty() {
                normalized.push('_');
            }
            normalized.push(char::from(byte));
            separator = false;
        } else {
            separator = true;
        }
    }
    match normalized.as_str() {
        "ascii" => Some("ascii"),
        "latin_1" => Some("iso8859-1"),
        "cp1252" => Some("cp1252"),
        "utf_8" => Some("utf-8"),
        "utf_8_sig" => Some("utf-8-sig"),
        "utf_16" => Some("utf-16"),
        "utf_16_be" => Some("utf-16-be"),
        "utf_16_le" => Some("utf-16-le"),
        _ => alias(&normalized).or_else(|| alias(&normalized.replace('.', "_"))),
    }
}

/// Python retries dotted names against aliases, but not canonical codec modules.
fn alias(name: &str) -> Option<&'static str> {
    match name {
        "646" | "ansi_x3.4_1968" | "ansi_x3.4_1986" | "ansi_x3_4_1968" | "cp367" | "csascii"
        | "ibm367" | "iso646_us" | "iso_646.irv_1991" | "iso_ir_6" | "us" | "us_ascii" => {
            Some("ascii")
        }
        "8859" | "cp819" | "csisolatin1" | "ibm819" | "iso8859" | "iso8859_1" | "iso_8859_1"
        | "iso_8859_1_1987" | "iso_ir_100" | "l1" | "latin" | "latin1" => Some("iso8859-1"),
        "1252" | "windows_1252" => Some("cp1252"),
        "cp65001" | "u8" | "utf" | "utf8" | "utf8_ucs2" | "utf8_ucs4" => Some("utf-8"),
        "u16" | "utf16" => Some("utf-16"),
        "unicodebigunmarked" | "utf_16be" => Some("utf-16-be"),
        "unicodelittleunmarked" | "utf_16le" => Some("utf-16-le"),
        _ => None,
    }
}
