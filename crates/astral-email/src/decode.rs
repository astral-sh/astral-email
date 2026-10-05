//! Unfolding and RFC 2047 decoding for header values.

use std::borrow::Cow;
use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};

/// An encoded header word could not be decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// A Base64 word has invalid padding or an incomplete group.
    InvalidBase64,
    /// The declared charset is unknown or unsupported.
    UnsupportedCharset(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase64 => formatter.write_str("invalid Base64 in encoded header word"),
            Self::UnsupportedCharset(name) => {
                write!(formatter, "unsupported header charset: {name}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Decode after unfolding continuation lines; ordinary UTF-8 remains borrowed.
///
/// Unlike Python's intermediate `raw-unicode-escape` representation, ordinary
/// Unicode text beside encoded words remains Unicode. Invalid raw UTF-8 and
/// malformed bytes in a supported charset become U+FFFD.
pub(crate) fn decode(raw: &[u8]) -> Result<Cow<'_, str>, DecodeError> {
    let text = String::from_utf8_lossy(raw);
    let unfolded = unfold(&text);
    if !has_encoded_word(&unfolded) {
        return Ok(match unfolded {
            Cow::Borrowed(_) => text,
            Cow::Owned(value) => Cow::Owned(value),
        });
    }

    let mut output = String::new();
    // Python validates every encoded payload before converting charsets.
    let mut charset_result = Ok(());
    let mut pending: Option<(&str, Vec<u8>)> = None;
    let mut previous_plain = false;
    let mut parts = parts(&unfolded).peekable();
    while let Some(part) = parts.next() {
        let word = match part {
            Part::Encoded(word) => word,
            Part::Plain(value) => {
                if pending.is_some()
                    && value.chars().all(is_whitespace)
                    && matches!(parts.peek(), Some(Part::Encoded(_)))
                {
                    continue;
                }
                charset_result = charset_result.and(flush(&mut pending, &mut output));
                if previous_plain {
                    output.push(' ');
                }
                output.push_str(value);
                previous_plain = true;
                continue;
            }
        };
        previous_plain = false;

        let bytes = match word.encoding {
            b'q' | b'Q' => decode_q(word.value),
            _ => decode_b(word.value)?,
        };
        if pending
            .as_ref()
            .is_some_and(|(name, _)| !name.eq_ignore_ascii_case(word.charset))
        {
            charset_result = charset_result.and(flush(&mut pending, &mut output));
        }
        if let Some((_, previous)) = &mut pending {
            previous.extend_from_slice(&bytes);
        } else {
            pending = Some((word.charset, bytes));
        }
    }
    charset_result.and(flush(&mut pending, &mut output))?;
    Ok(Cow::Owned(output))
}

/// Python checks for a marker before splitting lines; charsets can include LF.
fn has_encoded_word(value: &str) -> bool {
    let bytes = value.as_bytes();
    let Some(start) = value.find("=?") else {
        return false;
    };
    let mut charset = true;
    let mut payload_start = None;
    for offset in memchr::memchr2_iter(b'?', b'\n', &bytes[start + 2..]) {
        let index = start + 2 + offset;
        if bytes[index] == b'\n' {
            payload_start = None;
            continue;
        }
        if payload_start.is_some_and(|start| index >= start) && bytes.get(index + 1) == Some(&b'=')
        {
            return true;
        }
        if charset
            && matches!(bytes.get(index + 1), Some(b'q' | b'Q' | b'b' | b'B'))
            && bytes.get(index + 2) == Some(&b'?')
        {
            payload_start.get_or_insert(index + 3);
        }
        charset = bytes[index - 1] == b'=';
    }
    false
}

enum Part<'a> {
    Plain(&'a str),
    Encoded(Word<'a>),
}

/// Split like Python's `str.splitlines`, then recognize words within each line.
fn parts(value: &str) -> impl Iterator<Item = Part<'_>> {
    value
        .split([
            '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}',
            '\u{2029}',
        ])
        .flat_map(|line| {
            let mut remaining = line.trim_start_matches(is_whitespace);
            std::iter::from_fn(move || {
                if remaining.is_empty() {
                    return None;
                }
                if let Some(word) = next_word(remaining) {
                    if word.start == 0 {
                        remaining = &remaining[word.end..];
                        Some(Part::Encoded(word))
                    } else {
                        let plain = &remaining[..word.start];
                        remaining = &remaining[word.start..];
                        Some(Part::Plain(plain))
                    }
                } else {
                    Some(Part::Plain(std::mem::take(&mut remaining)))
                }
            })
        })
}

/// Replace a folded newline and its following indentation with one space.
fn unfold(value: &str) -> Cow<'_, str> {
    let bytes = value.as_bytes();
    let mut output = None;
    let mut copied = 0;
    let mut cursor = 0;
    while let Some(offset) = value[cursor..].find('\n') {
        let newline = cursor + offset;
        cursor = newline + 1;
        if !matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
            continue;
        }
        let end = newline - usize::from(newline > 0 && bytes[newline - 1] == b'\r');
        let output = output.get_or_insert_with(|| String::with_capacity(value.len()));
        output.push_str(&value[copied..end]);
        output.push(' ');
        while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
            cursor += 1;
        }
        copied = cursor;
    }
    match output {
        Some(mut output) => {
            output.push_str(&value[copied..]);
            Cow::Owned(output)
        }
        None => Cow::Borrowed(value),
    }
}

struct Word<'a> {
    start: usize,
    end: usize,
    charset: &'a str,
    encoding: u8,
    value: &'a str,
}

/// Find Python's next complete encoded word within a single line.
fn next_word(value: &str) -> Option<Word<'_>> {
    let mut cursor = 0;
    while let Some(offset) = value[cursor..].find("=?") {
        let start = cursor + offset;
        cursor = start + 2;
        let charset_end = cursor + memchr::memchr(b'?', &value.as_bytes()[cursor..])?;
        let tail = &value.as_bytes()[charset_end + 1..];
        if !matches!(tail.first(), Some(b'q' | b'Q' | b'b' | b'B')) || tail.get(1) != Some(&b'?') {
            continue;
        }
        let encoded_start = charset_end + 3;
        let encoded_end = encoded_start + value[encoded_start..].find("?=")?;
        return Some(Word {
            start,
            end: encoded_end + 2,
            charset: &value[cursor..charset_end],
            encoding: tail[0],
            value: &value[encoded_start..encoded_end],
        });
    }
    None
}

/// Python also treats these four ASCII separators as whitespace.
fn is_whitespace(character: char) -> bool {
    character.is_whitespace() || matches!(character, '\u{1c}'..='\u{1f}')
}

/// Convert non-ASCII encoded payload text as Python's `raw-unicode-escape` does.
fn payload_bytes(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.len());
    for character in value.chars() {
        if u32::from(character) <= 255 {
            bytes.push(character as u8);
        } else if u32::from(character) <= 0xffff {
            bytes.extend_from_slice(format!("\\u{:04x}", u32::from(character)).as_bytes());
        } else {
            bytes.extend_from_slice(format!("\\U{:08x}", u32::from(character)).as_bytes());
        }
    }
    bytes
}

/// Q words preserve malformed hex escapes; underscores represent spaces.
fn decode_q(value: &str) -> Vec<u8> {
    let mut bytes = payload_bytes(value);
    let mut read = 0;
    let mut written = 0;
    while read < bytes.len() {
        let decoded = if bytes[read] == b'=' && read + 2 < bytes.len() {
            char::from(bytes[read + 1])
                .to_digit(16)
                .zip(char::from(bytes[read + 2]).to_digit(16))
                .map(|(high, low)| (high * 16 + low) as u8)
        } else {
            None
        };
        bytes[written] = if let Some(byte) = decoded {
            read += 3;
            byte
        } else {
            let byte = bytes[read];
            read += 1;
            if byte == b'_' { b' ' } else { byte }
        };
        written += 1;
    }
    bytes.truncate(written);
    bytes
}

/// Match Python's permissive Base64 filtering and its original-length padding.
fn decode_b(value: &str) -> Result<Vec<u8>, DecodeError> {
    const ENGINE: GeneralPurpose = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
    );
    let padding = (4 - value.chars().count() % 4) % 4;
    let mut filtered = Vec::new();
    let mut group = 0;
    let mut pads = 0;
    for byte in payload_bytes(value)
        .into_iter()
        .chain(std::iter::repeat_n(b'=', padding))
    {
        if byte == b'=' {
            if group >= 2 {
                pads += 1;
                if group + pads == 4 {
                    filtered.extend(std::iter::repeat_n(b'=', 4 - group));
                    group = 0;
                    break;
                }
            }
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/') {
            filtered.push(byte);
            group = (group + 1) % 4;
            pads = 0;
        }
    }
    if group != 0 {
        return Err(DecodeError::InvalidBase64);
    }
    ENGINE
        .decode(filtered)
        .map_err(|_| DecodeError::InvalidBase64)
}

/// Decode complete runs together so adjacent words can split a multibyte character.
fn flush(pending: &mut Option<(&str, Vec<u8>)>, output: &mut String) -> Result<(), DecodeError> {
    if let Some((name, bytes)) = pending.take() {
        output.push_str(&decode_charset(name, &bytes)?);
    }
    Ok(())
}

/// Preserve Python's ASCII and Latin-1 meanings instead of WHATWG aliasing.
fn decode_charset<'a>(name: &str, bytes: &'a [u8]) -> Result<Cow<'a, str>, DecodeError> {
    let normalized = crate::charset::lookup(name)
        .ok_or_else(|| DecodeError::UnsupportedCharset(name.to_owned()))?;
    let decoded = match normalized {
        "ascii" => {
            if bytes.is_ascii() {
                Cow::Borrowed(str::from_utf8(bytes).expect("ASCII is valid UTF-8"))
            } else {
                Cow::Owned(
                    bytes
                        .iter()
                        .map(|&byte| {
                            if byte.is_ascii() {
                                char::from(byte)
                            } else {
                                '\u{fffd}'
                            }
                        })
                        .collect(),
                )
            }
        }
        "iso8859-1" => encoding_rs::mem::decode_latin1(bytes),
        "utf-8" => String::from_utf8_lossy(bytes),
        "utf-8-sig" => {
            String::from_utf8_lossy(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes))
        }
        "utf-16" => {
            let (encoding, bytes) = if let Some(bytes) = bytes.strip_prefix(b"\xfe\xff") {
                (encoding_rs::UTF_16BE, bytes)
            } else if let Some(bytes) = bytes.strip_prefix(b"\xff\xfe") {
                (encoding_rs::UTF_16LE, bytes)
            } else {
                (
                    if cfg!(target_endian = "big") {
                        encoding_rs::UTF_16BE
                    } else {
                        encoding_rs::UTF_16LE
                    },
                    bytes,
                )
            };
            encoding.decode_without_bom_handling(bytes).0
        }
        "utf-16-be" | "utf-16-le" => {
            let encoding = if normalized == "utf-16-be" {
                encoding_rs::UTF_16BE
            } else {
                encoding_rs::UTF_16LE
            };
            encoding.decode_without_bom_handling(bytes).0
        }
        "cp1252" => {
            let decoded = encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(bytes)
                .0;
            if decoded.contains(['\u{81}', '\u{8d}', '\u{8f}', '\u{90}', '\u{9d}']) {
                Cow::Owned(
                    decoded
                        .chars()
                        .map(|character| match character {
                            '\u{81}' | '\u{8d}' | '\u{8f}' | '\u{90}' | '\u{9d}' => '\u{fffd}',
                            character => character,
                        })
                        .collect(),
                )
            } else {
                decoded
            }
        }
        _ => return Err(DecodeError::UnsupportedCharset(name.to_owned())),
    };
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, decode};
    use std::borrow::Cow;

    #[test]
    fn ordinary_values_remain_borrowed() {
        for value in [
            "",
            "plain value",
            "  café\t ",
            "=?utf-8?x?literal?=",
            "=?utf-8?q?unfinished",
        ] {
            assert!(
                matches!(decode(value.as_bytes()).unwrap(), Cow::Borrowed(text) if text == value)
            );
        }
        assert_eq!(decode(b"bad \xff").unwrap(), "bad \u{fffd}");
    }

    #[test]
    fn unfold_continuations() {
        assert_eq!(decode(b"one\r\n \t two\n\tthree").unwrap(), "one two three");
        assert_eq!(decode(b"one\n\ttwo  ").unwrap(), "one two  ");
        assert_eq!(decode(b"one\r two").unwrap(), "one\r two");
    }

    #[test]
    fn python_encoded_words() {
        // Expected strings use decode_header, then each declared codec with errors="replace".
        for (value, expected) in [
            ("=?utf-8?q?hello_world?=", "hello world"),
            ("x=?utf-8?q?y?=z", "xyz"),
            (" =?utf-8?q?a?=  =?UTF-8?q?b?= ", "ab "),
            ("=?utf-8?q?=C3?= =?utf-8?q?=A9?=", "é"),
            ("=?utf-8?q?=C3=A9?= =?iso-8859-1?q?=E9?=", "éé"),
            ("=?utf-8?q?=GG_=4_=?=", "=GG =4 ="),
            ("=?utf-8?q?=5F?=", "_"),
            ("=?utf-8?b?YQ?=", "a"),
            ("=?utf-8?b?YWJj?=", "abc"),
            ("=?utf-8?b?YR==?=", "a"),
            ("=?utf-8?b?Y=Q==?=", "a"),
            ("=?utf-8?b?YQ===ignored?=", "a"),
            ("=?utf-8?b?====?=", ""),
            ("=?utf-8?b??=", ""),
            ("=?ascii?Q?=FF?=", "\u{fffd}"),
            ("=?iso-8859-1?Q?=80?=", "\u{80}"),
            ("=?windows-1252?Q?=80?=", "€"),
            ("=?utf-16?B?//5oAGkA?=", "hi"),
            ("=?utf-16-be?B?AGgAaQ==?=", "hi"),
        ] {
            assert_eq!(decode(value.as_bytes()).unwrap(), expected, "{value}");
        }
    }

    #[test]
    fn malformed_base64_matches_python_errors() {
        for value in ["=?utf-8?b?Y?=", "=?utf-8?b?Y!Q?=", "=?utf-8?b?Y!Q=?="] {
            assert_eq!(
                decode(value.as_bytes()),
                Err(DecodeError::InvalidBase64),
                "{value}"
            );
        }
        assert_eq!(
            decode(b"=?not-a-charset?q?text?="),
            Err(DecodeError::UnsupportedCharset("not-a-charset".to_owned()))
        );
        for charset in ["utf-7", "iso-8859-9", "shift_jis"] {
            assert_eq!(
                decode(format!("=?{charset}?q?text?=").as_bytes()),
                Err(DecodeError::UnsupportedCharset(charset.to_owned()))
            );
        }
    }

    #[test]
    fn base64_errors_precede_charset_errors() {
        for value in [
            "=?unknown?q?a?= text =?utf-8?b?Y?=",
            "=?unknown?q?a?= =?utf-8?q?b?= =?utf-8?b?Y?=",
            "=?unknown?b?YQ?= text =?another?q?b?= =?utf-8?b?Y?=",
            "=?utf-8?b?Y?= text =?unknown?q?a?=",
        ] {
            assert_eq!(
                decode(value.as_bytes()),
                Err(DecodeError::InvalidBase64),
                "{value}"
            );
        }
        assert_eq!(
            decode(b"=?unknown?q?a?= text =?another?q?b?="),
            Err(DecodeError::UnsupportedCharset("unknown".to_owned()))
        );
    }

    #[test]
    fn python_codec_fixtures() {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/decode.json")).unwrap();
        let cases = fixtures["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 60);
        for case in cases {
            let value = case["value"].as_str().unwrap();
            assert_eq!(
                decode(value.as_bytes()).unwrap(),
                case["expected"].as_str().unwrap(),
                "{value}"
            );
        }
    }

    #[test]
    fn unicode_and_folding_convenience_policy() {
        assert_eq!(decode("café =?utf-8?q?ok?=".as_bytes()).unwrap(), "café ok");
        assert_eq!(decode(b"=?utf-8?q?a?=\r\n\t=?utf-8?q?b?=").unwrap(), "ab");
        assert_eq!(decode(b"=?utf-8?q?a?=\r\n\tb").unwrap(), "a b");
    }

    #[test]
    fn python_line_separators() {
        for separator in [
            '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}', '\u{2029}',
        ] {
            let plain = format!("before{separator} after");
            assert!(
                matches!(decode(plain.as_bytes()).unwrap(), Cow::Borrowed(value) if value == plain)
            );
            for (value, expected) in [
                (format!("=?utf-8?q?hello?={separator}"), "hello"),
                (format!("=?utf-8?q?hello?={separator}  world"), "helloworld"),
                (format!("prefix{separator}=?utf-8?q?hello?="), "prefixhello"),
                (
                    format!("before{separator} after =?utf-8?q?end?="),
                    "before after end",
                ),
                (format!("=?utf-8?q?=C3?={separator}=?utf-8?q?=A9?="), "é"),
                (
                    format!("=?utf-8?q?hello{separator}world?="),
                    "=?utf-8?q?hello world?=",
                ),
                (
                    format!("=?utf{separator}-8?q?hello?="),
                    "=?utf -8?q?hello?=",
                ),
            ] {
                assert_eq!(decode(value.as_bytes()).unwrap(), expected, "{value:?}");
            }
        }
        assert_eq!(
            decode(b"=?utf-\n8?q?hello?=").unwrap(),
            "=?utf- 8?q?hello?="
        );
        assert_eq!(
            decode(b"=?utf-8?q?hello\nworld?=").unwrap(),
            "=?utf-8?q?hello\nworld?="
        );
        assert_eq!(
            decode(b"=?utf-8?q?outer =?\nutf-8?q?inner?=").unwrap(),
            "=?utf-8?q?outer =? utf-8?q?inner?="
        );
    }
}
