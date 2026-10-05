# Decoded header values

`Header::raw_value` preserves bytes under the [raw parsing contract](conformance.md).
`Header::decoded_value` converts them to text:

1. Decode ordinary bytes as UTF-8, replacing invalid sequences with U+FFFD.
2. Replace CRLF or LF followed by spaces or tabs with one space.
3. Decode RFC 2047 Q/B words using Python's `email.header.decode_header` syntax
   and convert supported charsets with replacement for malformed bytes.

With a complete encoded-word marker, values are split into lines as Python does,
including vertical tab, form feed, and Unicode line separators. Leading whitespace
on each line is stripped; consecutive ordinary pieces are joined with one space.
Whitespace between adjacent encoded words is ignored. Words with the same charset label
(ignoring ASCII case) are joined before conversion, allowing split multibyte
characters. Different charset aliases are decoded separately.

Unchanged ordinary UTF-8 values borrow the input. Q words replace underscores
with spaces and decode hex escapes; malformed escapes remain literal. B words
use Python's permissive Base64 filtering and missing-padding handling. Incomplete
markers and encodings other than Q or B remain literal.

Invalid Base64 returns `DecodeError::InvalidBase64`, taking precedence over
charset errors. Otherwise, the first unsupported charset returns
`DecodeError::UnsupportedCharset`.

Supported codecs are ASCII, Latin-1, Windows-1252, UTF-8, `utf-8-sig`, `utf-16`,
`utf-16-le`, and `utf-16-be`. Python's case and separator normalization applies:
`us-ascii`, `iso-8859-1`, `cp1252`, `utf8`, `utf 8`, `utf--8`, and `windows.1252`
are accepted; `utf.8` is not. Other codecs, including UTF-7, are unsupported.

UTF-16 consumes an initial BOM and otherwise uses native byte order. Explicit
byte-order labels preserve a BOM as U+FEFF; UTF-8 removes one only for
`utf-8-sig`. ASCII and Latin-1 retain their Python meanings. Undefined Windows-1252
bytes become U+FFFD.

Unlike `compat32` field lookup, this API decodes encoded words. It unfolds first
and preserves ordinary Unicode beside them: `café =?utf-8?q?ok?=` becomes `café ok`,
without Python's intermediate `raw-unicode-escape` conversion of ordinary text.

## Checking the decoder

Run with CPython 3.12.13 on a little-endian host:

```console
python scripts/generate_decode_fixtures.py --check
cargo test -p astral-mail-headers --lib decode::tests
cargo build -p astral-mail-headers --example decode_inspect --locked
python scripts/check_decode.py target/debug/examples/decode_inspect
```

Omit `--check` to regenerate the [codec fixtures](../crates/astral-mail-headers/tests/fixtures/decode.json).
The differential check compares ASCII inputs using supported codecs. Both adapters
parse an `X:` header to apply the same trimming; Python then unfolds its value,
calls `decode_header`, and converts the parts with `errors="replace"`.
`generate_decode_fixtures.py --stdin` and `decode_inspect` accept JSON lines with a
`value` string and return decoded text or an error code.
