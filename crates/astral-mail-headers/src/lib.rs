//! A high-performance email header parser designed for Python packaging.
//!
//! Header structure and recovery follow Python's `BytesHeaderParser` with the
//! `compat32` policy. Values retain folding and encoded words; the body is opaque.
//!
//! ```
//! use astral_mail_headers::Message;
//!
//! let message = Message::parse(b"Name: example\nRequires-Dist: requests>=2\n\nDescription\n");
//! assert_eq!(message.first("name").unwrap().raw_value(), b"example");
//! assert_eq!(message.all("Requires-Dist").count(), 1);
//! assert_eq!(message.body(), b"Description\n");
//! ```

use std::borrow::Cow;

use memchr::memchr2;

mod charset;
mod decode;

pub use decode::DecodeError;

/// A recoverable header error reported by Python's `compat32` parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defect {
    /// A continuation line has no preceding field.
    FirstHeaderLineIsContinuation,
    /// A non-header line begins the body without an empty separator line.
    MissingHeaderBodySeparator,
    /// A field has an empty name.
    InvalidHeader,
    /// An envelope line occurs between header lines.
    MisplacedEnvelopeHeader,
}

/// A header whose name and value borrow from the source.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Header<'a> {
    name: &'a [u8],
    value: &'a [u8],
}

impl<'a> Header<'a> {
    /// The original spelling of the ASCII field name.
    #[inline]
    pub fn name(&self) -> &'a str {
        std::str::from_utf8(self.name).expect("header names contain only printable ASCII")
    }

    /// The value returned by Python's `raw_items`, as bytes.
    ///
    /// Leading spaces, tabs and line endings and trailing line endings are
    /// removed. Interior folding, encoded words and non-ASCII bytes are retained.
    pub fn raw_value(&self) -> &'a [u8] {
        self.value
    }

    /// Unfold and decode RFC 2047 words for display or metadata extraction.
    ///
    /// This is an explicit conversion; Python's `compat32` raw values retain
    /// folding and encoded words. Unchanged ordinary UTF-8 values remain borrowed.
    /// Invalid text becomes U+FFFD. See the [decoding policy] for supported charsets.
    ///
    /// [decoding policy]: https://github.com/astral-sh/astral-mail-headers/blob/main/docs/decoding.md
    ///
    /// # Errors
    ///
    /// Returns an error for invalid Base64 or an unsupported declared charset.
    #[inline]
    pub fn decoded_value(&self) -> Result<Cow<'a, str>, DecodeError> {
        decode::decode(self.value)
    }
}

impl std::fmt::Debug for Header<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Header")
            .field("name", &self.name())
            .field("value", &self.value)
            .finish()
    }
}

/// Ordered headers and an opaque body parsed with Python's `compat32` recovery.
#[derive(Debug, Clone)]
pub struct Message<'a> {
    headers: Vec<Header<'a>>,
    body: Cow<'a, [u8]>,
    unix_from: Option<&'a [u8]>,
    defects: Vec<Defect>,
}

impl<'a> Message<'a> {
    /// Parse headers, preserving duplicates and recording recoverable defects.
    ///
    /// Parsing stops at the first empty line or non-header line. The body is
    /// borrowed, except when Python's recovery moves a trailing envelope line
    /// into the body across an empty separator.
    pub fn parse(source: &'a [u8]) -> Self {
        let mut headers = Vec::with_capacity(8);
        let mut defects = Vec::new();
        let mut unix_from = None;
        let mut pending: Option<(&[u8], usize, usize)> = None;
        let mut envelope: Option<(usize, usize)> = None;
        let mut position = 0;

        while position < source.len() {
            let start = position;
            let content_end = memchr2(b'\r', b'\n', &source[start..])
                .map_or(source.len(), |offset| start + offset);
            let mut end = content_end;
            if end < source.len() {
                end += 1;
                if source[content_end] == b'\r' && source.get(end) == Some(&b'\n') {
                    end += 1;
                }
            }
            let line = &source[start..content_end];
            if line.is_empty() {
                position = end;
                break;
            }
            if matches!(line[0], b' ' | b'\t') {
                position = end;
                if envelope.take().is_some() {
                    defects.push(Defect::MisplacedEnvelopeHeader);
                }
                if let Some((_, _, value_end)) = &mut pending {
                    *value_end = content_end;
                } else {
                    defects.push(Defect::FirstHeaderLineIsContinuation);
                }
                continue;
            }
            let is_envelope = line.starts_with(b"From ");
            let colon = if is_envelope {
                None
            } else {
                line.iter()
                    .position(|byte| !(b'!'..=b'~').contains(byte) || *byte == b':')
                    .filter(|index| line[*index] == b':')
            };
            if !is_envelope && colon.is_none() {
                // Python gathers lines before processing individual headers.
                defects.insert(0, Defect::MissingHeaderBodySeparator);
                break;
            }
            position = end;
            if envelope.take().is_some() {
                defects.push(Defect::MisplacedEnvelopeHeader);
            }

            if let Some((name, value_start, value_end)) = pending.take() {
                headers.push(header(name, &source[value_start..value_end]));
            }
            if is_envelope {
                if start == 0 {
                    unix_from = Some(line);
                } else {
                    envelope = Some((start, end));
                }
            } else if let Some(colon) = colon {
                if colon == 0 {
                    defects.push(Defect::InvalidHeader);
                } else {
                    let name = &line[..colon];
                    pending = Some((name, start + colon + 1, content_end));
                }
            }
        }
        if let Some((name, value_start, value_end)) = pending {
            headers.push(header(name, &source[value_start..value_end]));
        }
        let body = if let Some((start, end)) = envelope {
            if end == position {
                Cow::Borrowed(&source[start..])
            } else {
                let mut body = Vec::with_capacity(end - start + source.len() - position);
                body.extend_from_slice(&source[start..end]);
                body.extend_from_slice(&source[position..]);
                Cow::Owned(body)
            }
        } else {
            Cow::Borrowed(&source[position..])
        };
        Self {
            headers,
            body,
            unix_from,
            defects,
        }
    }

    /// Headers in source order, including repeated names.
    pub fn headers(&self) -> &[Header<'a>] {
        &self.headers
    }

    /// Find the first header with this name, ignoring ASCII case.
    pub fn first(&self, name: &str) -> Option<&Header<'a>> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name.as_bytes()))
    }

    /// Iterate over matching headers in source order, ignoring ASCII case.
    pub fn all<'m>(&'m self, name: &'m str) -> impl Iterator<Item = &'m Header<'a>> {
        self.headers
            .iter()
            .filter(move |header| header.name.eq_ignore_ascii_case(name.as_bytes()))
    }

    /// Body bytes, without MIME parsing, decoding or newline normalization.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// The initial `From ` envelope line, without its line ending.
    pub fn unix_from(&self) -> Option<&'a [u8]> {
        self.unix_from
    }

    /// Recoverable errors in the order reported by Python.
    pub fn defects(&self) -> &[Defect] {
        &self.defects
    }
}

/// Trim the start of a field whose final line ending has already been excluded.
fn header<'a>(name: &'a [u8], mut value: &'a [u8]) -> Header<'a> {
    while matches!(value.first(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        value = &value[1..];
    }
    Header { name, value }
}
