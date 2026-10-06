# Conformance

Parsing follows CPython 3.12.13's
`email.parser.BytesHeaderParser(policy=email.policy.compat32).parsebytes`.
[Core metadata](https://packaging.python.org/en/latest/specifications/core-metadata/)
and [WHEEL](https://packaging.python.org/en/latest/specifications/binary-distribution-format/#file-contents)
use this header format; metadata field validation remains the caller's responsibility.

## Raw parsing

Headers retain their names, order, and duplicates. Values remove leading spaces,
tabs, and line endings and trailing line endings; all remaining bytes, including
folding, encoded words, and invalid text, are preserved. LF, CRLF, and CR line
endings are accepted. Malformed headers follow Python's recovery and defect
ordering; defect messages and attached line data are omitted.

The initial `From ` envelope line is stored separately. A trailing `From ` line
after headers is recovered into the body, omitting the empty separator if present.
This can require an owned body; ordinary bodies borrow the source unchanged.

## Decoding and exclusions

Raw parsing leaves text conversion to [`Header::decoded_value`](decoding.md).
That accessor differs from [Python's `compat32` field lookup](https://docs.python.org/3/library/email.policy.html#email.policy.Compat32).
MIME parsing, body decoding, address and date grammars, writing, and mutation are
outside this parser's scope.

## Resource behavior

Header storage grows with the number of fields and defects; lookups scan the
header list. There are no built-in size or field-count limits, so callers must
bound input reads to their resource budget.

See [Contributing](../CONTRIBUTING.md) for fixture generation and differential tests.
