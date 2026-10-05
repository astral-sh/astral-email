//! Read-only parsing of Python packaging headers.
//!
//! Header structure and recovery follow Python's `BytesHeaderParser` with the
//! `compat32` policy. Values retain folding and encoded words; the body is opaque.
//!
//! ```
//! use astral_email::Message;
//!
//! let message = Message::parse(b"Name: example\nRequires-Dist: requests>=2\n\nDescription\n");
//! assert_eq!(message.first("name").unwrap().raw_value(), b"example");
//! assert_eq!(message.all("Requires-Dist").count(), 1);
//! assert_eq!(message.body(), b"Description\n");
//! ```

use std::borrow::Cow;
use std::ops::Range;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header<'a> {
    name: &'a str,
    value: &'a [u8],
}

impl<'a> Header<'a> {
    /// The original spelling of the ASCII field name.
    pub fn name(&self) -> &'a str {
        self.name
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
    /// [decoding policy]: https://github.com/viarius-experiments/astral-email/blob/main/docs/decoding.md
    ///
    /// # Errors
    ///
    /// Returns an error for invalid Base64 or an unsupported declared charset.
    pub fn decoded_value(&self) -> Result<Cow<'a, str>, DecodeError> {
        decode::decode(self.value)
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
        let mut headers = Vec::new();
        let mut defects = Vec::new();
        let (body, unix_from) = parse(
            source,
            |offsets| headers.push(offsets.get(source)),
            &mut defects,
        );
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
            .find(|header| header.name.eq_ignore_ascii_case(name))
    }

    /// Iterate over matching headers in source order, ignoring ASCII case.
    pub fn all<'m>(&'m self, name: &'m str) -> impl Iterator<Item = &'m Header<'a>> {
        self.headers
            .iter()
            .filter(move |header| header.name.eq_ignore_ascii_case(name))
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

/// Header storage retained between calls to [`Parser::parse`].
///
/// Capacity follows the largest message parsed and is released when the parser
/// is dropped. A parsed view borrows the storage until its last use.
#[derive(Debug, Default)]
pub struct Parser {
    headers: Vec<HeaderOffsets>,
    defects: Vec<Defect>,
}

impl Parser {
    /// Parse a message while reusing the header and defect allocations.
    ///
    /// ```
    /// use astral_email::Parser;
    /// let mut parser = Parser::default();
    /// assert_eq!(parser.parse(b"Name: first").first("Name").unwrap().raw_value(), b"first");
    /// assert_eq!(parser.parse(b"Name: next").first("Name").unwrap().raw_value(), b"next");
    /// ```
    pub fn parse<'p, 's>(&'p mut self, source: &'s [u8]) -> MessageView<'p, 's> {
        self.headers.clear();
        self.defects.clear();
        let (body, unix_from) = parse(
            source,
            |header| self.headers.push(header),
            &mut self.defects,
        );
        MessageView {
            source,
            headers: &self.headers,
            body,
            unix_from,
            defects: &self.defects,
        }
    }
}

/// Immutable message data backed by a reusable [`Parser`].
#[derive(Debug)]
pub struct MessageView<'p, 's> {
    source: &'s [u8],
    headers: &'p [HeaderOffsets],
    body: Cow<'s, [u8]>,
    unix_from: Option<&'s [u8]>,
    defects: &'p [Defect],
}

impl<'s> MessageView<'_, 's> {
    /// Headers in source order, including repeated names.
    pub fn headers(&self) -> impl ExactSizeIterator<Item = Header<'s>> + '_ {
        self.headers.iter().map(|header| header.get(self.source))
    }

    /// Find the first header with this name, ignoring ASCII case.
    pub fn first(&self, name: &str) -> Option<Header<'s>> {
        self.headers
            .iter()
            .find(|header| self.source[header.name.clone()].eq_ignore_ascii_case(name.as_bytes()))
            .map(|header| header.get(self.source))
    }

    /// Iterate over matching headers in source order, ignoring ASCII case.
    pub fn all<'m>(&'m self, name: &'m str) -> impl Iterator<Item = Header<'s>> + 'm {
        self.headers
            .iter()
            .filter(move |header| {
                self.source[header.name.clone()].eq_ignore_ascii_case(name.as_bytes())
            })
            .map(|header| header.get(self.source))
    }

    /// Body bytes, without MIME parsing, decoding or newline normalization.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// The initial `From ` envelope line, without its line ending.
    pub fn unix_from(&self) -> Option<&'s [u8]> {
        self.unix_from
    }

    /// Recoverable errors in the order reported by Python.
    pub fn defects(&self) -> &[Defect] {
        self.defects
    }
}

#[derive(Debug)]
struct HeaderOffsets {
    name: Range<usize>,
    value: Range<usize>,
}

impl HeaderOffsets {
    /// Borrow the validated field name and trimmed value from the source.
    fn get<'a>(&self, source: &'a [u8]) -> Header<'a> {
        Header {
            name: str::from_utf8(&source[self.name.clone()])
                .expect("header names contain only printable ASCII"),
            value: &source[self.value.clone()],
        }
    }
}

/// Parse shared structure into borrowed headers or reusable offset records.
fn parse<'a>(
    source: &'a [u8],
    mut push_header: impl FnMut(HeaderOffsets),
    defects: &mut Vec<Defect>,
) -> (Cow<'a, [u8]>, Option<&'a [u8]>) {
    let mut unix_from = None;
    let mut pending: Option<(Range<usize>, usize, usize)> = None;
    let mut envelope: Option<(usize, usize)> = None;
    let mut position = 0;
    let mut line_number = 0;

    while position < source.len() {
        let start = position;
        let remaining = &source[start..];
        let empty = matches!(remaining[0], b'\r' | b'\n');
        let continuation = matches!(remaining[0], b' ' | b'\t');
        let is_envelope = remaining.starts_with(b"From ");
        let colon = if empty || continuation || is_envelope {
            None
        } else {
            remaining
                .iter()
                .position(|byte| !(b'!'..=b'~').contains(byte) || *byte == b':')
                .filter(|index| remaining[*index] == b':')
        };
        if !empty && !continuation && !is_envelope && colon.is_none() {
            // Python gathers lines before processing individual headers.
            defects.insert(0, Defect::MissingHeaderBodySeparator);
            break;
        }
        let value_start = colon.map_or(start, |colon| start + colon + 1);
        let content_end = memchr2(b'\r', b'\n', &source[value_start..])
            .map_or(source.len(), |offset| value_start + offset);
        let mut end = content_end;
        if end < source.len() {
            end += 1;
            if source[content_end] == b'\r' && source.get(end) == Some(&b'\n') {
                end += 1;
            }
        }
        let line = &source[start..content_end];
        if empty {
            position = end;
            break;
        }
        position = end;
        if envelope.take().is_some() {
            defects.push(Defect::MisplacedEnvelopeHeader);
        }

        if continuation {
            if let Some((_, _, value_end)) = &mut pending {
                *value_end = end;
            } else {
                defects.push(Defect::FirstHeaderLineIsContinuation);
            }
        } else {
            if let Some((name, value_start, value_end)) = pending.take() {
                push_header(header(source, name, value_start..value_end));
            }
            if is_envelope {
                if line_number == 0 {
                    unix_from = Some(line);
                } else {
                    envelope = Some((start, end));
                }
            } else if let Some(colon) = colon {
                if colon == 0 {
                    defects.push(Defect::InvalidHeader);
                } else {
                    pending = Some((start..start + colon, start + colon + 1, end));
                }
            }
        }
        line_number += 1;
    }
    if let Some((name, value_start, value_end)) = pending {
        push_header(header(source, name, value_start..value_end));
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
    (body, unix_from)
}

/// Apply `compat32.header_source_parse` trimming to a contiguous field value.
fn header(source: &[u8], name: Range<usize>, mut value: Range<usize>) -> HeaderOffsets {
    while value.start < value.end && matches!(source[value.start], b' ' | b'\t' | b'\r' | b'\n') {
        value.start += 1;
    }
    while value.start < value.end && matches!(source[value.end - 1], b'\r' | b'\n') {
        value.end -= 1;
    }
    HeaderOffsets { name, value }
}
