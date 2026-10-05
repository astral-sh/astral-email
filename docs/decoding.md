# Decoded header values

`Header::raw_value` preserves bytes. `Header::decoded_value` is a separate,
fallible convenience API:

1. Decode ordinary bytes as UTF-8, replacing invalid sequences with U+FFFD.
2. Unfold CRLF or LF followed by spaces or tabs into one space, removing the
   continuation indentation. Other whitespace is preserved.
3. Decode RFC 2047 Q and B words using Python's `email.header.decode_header`
   marker syntax. When encoded words occur, strip leading whitespace and ignore
   whitespace between adjacent encoded words. Combine adjacent words with the
   same charset before decoding their bytes.
4. Convert supported charsets with replacement for malformed byte sequences.

Ordinary values that need no transformation borrow from the input. Q words
replace underscores with spaces and decode hexadecimal escapes; malformed
escapes remain literal. B words follow Python's permissive Base64 filtering and
missing-padding handling. An incomplete Base64 group returns
`DecodeError::InvalidBase64`. An unknown or unsupported charset returns
`DecodeError::UnsupportedCharset`. Incomplete markers and markers with an
encoding other than Q or B remain literal.

The supported codecs are ASCII, Latin-1, Windows-1252, UTF-8, UTF-8 with a BOM,
and UTF-16 with optional explicit byte order. Matching ignores ASCII case and
treats underscores as hyphens. The accepted labels are:

| Codec | Labels |
| --- | --- |
| ASCII | `ascii`, `us-ascii`, `646`, `ansi-x3.4-1968`, `ansi-x3.4-1986`, `ansi-x3-4-1968`, `cp367`, `csascii`, `ibm367`, `iso646-us`, `iso-646.irv-1991`, `iso-ir-6`, `us` |
| Latin-1 | `latin-1`, `latin1`, `iso-8859-1`, `iso8859-1`, `iso8859`, `l1`, `8859`, `cp819`, `csisolatin1`, `ibm819`, `iso-8859-1-1987`, `iso-ir-100`, `latin` |
| Windows-1252 | `windows-1252`, `cp1252`, `1252` |
| UTF-8 | `utf-8`, `utf8`, `cp65001`, `u8`, `utf`, `utf8-ucs2`, `utf8-ucs4` |
| UTF-8 with BOM removal | `utf-8-sig` |
| UTF-16 | `utf-16`, `utf16`, `u16` |
| UTF-16 big endian | `utf-16-be`, `utf-16be`, `unicodebigunmarked` |
| UTF-16 little endian | `utf-16-le`, `utf-16le`, `unicodelittleunmarked` |

UTF-16 detects and removes an initial BOM; without one it uses native byte
order, as Python does. Explicit UTF-16 byte-order labels preserve a BOM as
U+FEFF. UTF-8 preserves a BOM unless the label is `utf-8-sig`. ASCII and Latin-1
retain their Python meanings rather than aliasing Windows-1252. Windows-1252's
five undefined bytes become U+FFFD. Other codecs, including UTF-7, are unsupported.

This API is not Python's `compat32` field lookup, which preserves encoded
words. It also deliberately unfolds before decoding and preserves ordinary
Unicode beside encoded words instead of exposing Python's intermediate
`raw-unicode-escape` byte conversion. For example, a folded plain value becomes
one line, and `café =?utf-8?q?ok?=` becomes `café ok`.

[Decoder fixtures](../crates/astral-email/tests/fixtures/decode.json) record
Python's decoded results for all 256 byte values in each supported single-byte
codec, plus malformed Unicode and BOM cases. The raw parser's separate
[compatibility contract](conformance.md) does not depend on codec support.
