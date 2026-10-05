# Decoded header values

`Header::raw_value` preserves bytes according to the [raw parsing contract](conformance.md).
`Header::decoded_value` is a separate, fallible convenience API:

1. Decode ordinary bytes as UTF-8, replacing invalid sequences with U+FFFD.
2. Unfold CRLF or LF followed by spaces or tabs into one space.
3. Decode RFC 2047 Q and B words using Python's `email.header.decode_header`
   marker syntax. With encoded words, strip leading whitespace and ignore
   whitespace between adjacent words. Join same-charset words before conversion
   so they can split a multibyte character.
4. Convert supported charsets with replacement for malformed bytes.

Unchanged ordinary UTF-8 values borrow the input. Q words replace underscores
with spaces and decode hexadecimal escapes; malformed escapes remain literal.
B words follow Python's permissive Base64 filtering and missing-padding handling.
Invalid Base64 returns `DecodeError::InvalidBase64`. Unknown or unsupported
charsets return `DecodeError::UnsupportedCharset`. Incomplete markers and
encodings other than Q or B remain literal.

Supported codecs are ASCII, Latin-1, Windows-1252, UTF-8, `utf-8-sig`, and UTF-16
with optional explicit byte order (`utf-16-le` or `utf-16-be`). Common Python
aliases such as `us-ascii`, `iso-8859-1`, `cp1252`, and `utf8` are accepted;
matching ignores ASCII case and treats underscores as hyphens. The decoder does
not implement Python's complete codec or alias registry. UTF-7 is unsupported.

UTF-16 removes an initial BOM and otherwise uses native byte order. Explicit
byte-order labels preserve a BOM as U+FEFF; UTF-8 removes one only for
`utf-8-sig`. ASCII and Latin-1 retain their Python meanings. Windows-1252's five
undefined bytes become U+FFFD.

This API is not `compat32` field lookup, which preserves encoded words. It also
deliberately unfolds before decoding and preserves ordinary Unicode beside
encoded words: `café =?utf-8?q?ok?=` becomes `café ok`, without Python's
intermediate `raw-unicode-escape` conversion of ordinary text.

## Checking the decoder

Run with CPython 3.12.13 on a little-endian host:

```console
python scripts/generate_decode_fixtures.py --check
cargo test -p astral-email --lib decode::tests
```

Omit `--check` to regenerate the 47 [codec fixtures](../crates/astral-email/tests/fixtures/decode.json).
They encode all 256 byte values for each supported single-byte codec and selected
malformed UTF-8/UTF-16 and BOM sequences. Expected strings come from
`email.header.decode_header`, then `bytes.decode(charset, errors="replace")`.

For bounded Q/B comparisons, `generate_decode_fixtures.py --stdin` and the
`decode_inspect` example accept JSON lines such as `{"value":"=?utf-8?B?YQ?="}`.
Both prepend `X:`, parse that message, and decode its first `X` value. This keeps
header trimming consistent. The Python wrapper unfolds the value first, calls
`decode_header`, then converts the returned parts with replacement. Output is
`{"value":"a"}` or an `error` code. Use ASCII inputs and the supported canonical
charset names; this comparison does not cover ordinary Unicode text or all
Python aliases. Folded inputs use LF or CRLF; bare-CR folding is outside this
decoded-value comparison.

The seeded driver compares 5,000 short values, including malformed Q/B payloads
and adjacent words split across multibyte characters:

```console
cargo build -p astral-email --example decode_inspect --locked
python scripts/check_decode.py target/debug/examples/decode_inspect
```
