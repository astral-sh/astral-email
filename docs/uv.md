# uv integration

[The adapter](uv-integration.patch) replaces mailparse in uv's `uv-pypi-types`
metadata reader and `uv-install-wheel` WHEEL reader. It preserves uv's field
selection, `UNKNOWN` filtering, repeated fields, WHEEL name casing and value
trimming, and description fallback.

## Test

Check out uv [`46b84fd0bfec23b72f29e8e2185ba68a65052f48`](https://github.com/astral-sh/uv/tree/46b84fd0bfec23b72f29e8e2185ba68a65052f48)
into `uv-source` at the repository root, then run:

```console
python3 scripts/test_uv.py
```

The script applies the adapter and runs the upstream library suites and
[adapter tests](../scripts/uv-tests/). It verifies that the upstream test modules
remain unchanged.

## Compatibility

Recovery follows Python's `BytesHeaderParser(policy=compat32)`. Compared with
mailparse:

- Colonless lines and invalid field names start the body.
- Leading continuations are recorded as defects; lone CR line endings are accepted.
- Invalid raw UTF-8 becomes U+FFFD during decoding instead of Latin-1 text.
- Unsupported encoded-word charsets and invalid Base64 return decoding errors
  instead of retaining the encoded text.

The adapter accepts parser defects and propagates decoding errors through uv's
metadata and wheel errors. The description remains bytes until uv validates its
UTF-8.

## Fixtures

The [fixture manifest](../crates/astral-mail-headers/tests/fixtures/uv/manifest.json)
records archive members, the source revision, content hashes, and expected Python
headers. uv's MIT and Apache licenses and Black's license are retained in the
[fixture directory](../crates/astral-mail-headers/tests/fixtures/uv/).

Regenerate from the pinned checkout:

```console
python3 scripts/update_uv_fixtures.py /path/to/pinned/uv
```

See [Benchmarks](performance.md) for benchmark commands.
