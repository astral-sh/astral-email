# Conformance

The parser targets CPython 3.12.13's
`email.parser.BytesHeaderParser(policy=email.policy.compat32).parsebytes`.
[Core metadata](https://packaging.python.org/en/latest/specifications/core-metadata/)
uses email headers and identifies `compat32` as its practical parsing standard.
[WHEEL](https://packaging.python.org/en/latest/specifications/binary-distribution-format/#file-contents)
uses the same basic header format. Metadata field validation remains the caller's
responsibility.

## Raw parsing

The fixtures compare ordered names and values from `raw_items()`, body bytes,
the initial Unix envelope line, and defect classes in Python's reported order.
They cover every byte in a field name, LF/CRLF/CR and mixed line endings,
continuations, empty and malformed fields, repeated names, envelope recovery,
EOF, and opaque bodies. There are no excluded cases in this corpus. This is a
focused compatibility suite, not a claim to implement every email RFC.

Python decodes input bytes as ASCII with `surrogateescape`. The generator reverses
that conversion to preserve bytes, including UTF-8 and invalid text. Parsing a
Unicode string with `Parser` is a different operation: stringifying a header from
`BytesHeaderParser` can replace non-ASCII bytes instead of decoding them as UTF-8.
The Rust raw API retains those bytes and leaves text decoding explicit.

Values follow `compat32.header_source_parse`: remove leading spaces, tabs and
line endings, and trailing line endings; preserve interior folding and encoded
words. Defects are `FirstHeaderLineIsContinuationDefect`,
`MissingHeaderBodySeparatorDefect`, `InvalidHeaderDefect`, and
`MisplacedEnvelopeHeaderDefect`. Python's defect messages and attached line data
are not part of the comparison.

A trailing `From ` line after headers is recovered into the body. If an empty
separator follows it, Python omits that separator when joining the recovered
line to the remaining body. This recovery can require an owned body; ordinary
bodies borrow the source. The generator reads Python's private `_payload` because
the public `get_payload()` may replace bytes or decode a transfer encoding.

## Reproducing the corpus

Run with CPython 3.12.13:

```console
python scripts/generate_conformance.py
python scripts/generate_conformance.py --check
cargo test -p astral-email --test python
```

`crates/astral-email/tests/fixtures/python.json` records the parser, policy and
Python version. Each case has `name`, `input_hex`, ordered `headers` with
`name_hex` and `value_hex`, `body_hex`, `unix_from_hex`, and `defects`.
`body_offset` is the start of the body when it equals a source suffix, or `null`
for recovery that joins noncontiguous bytes.

For differential checks, both `scripts/generate_conformance.py --stdin` and the
Rust `inspect` example accept JSON lines containing `{"input_hex": "..."}` and
emit the same parsed fields, without the case name or input. Fixtures contain
generated test data; generation and checking require no network access.

## Decoding and exclusions

Raw parsing does not unfold values or decode RFC 2047 words. Those operations
belong to the separate decoded-value accessor; its text and charset policy is
not a promise that it equals Python's `compat32` header string conversion.
See [Python's policy documentation](https://docs.python.org/3/library/email.policy.html#email.policy.Compat32)
for the distinction between source parsing and value retrieval.

The body is opaque. MIME trees, body transfer decoding, attachments, address and
date grammars, writing, and mutation are outside this parser's scope.
