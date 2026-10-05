# Fuzzing

Five targets cover parsing and decoding:

| Target | Checks |
| --- | --- |
| `reader` | Borrowed field ranges, header order, valid names, envelope position, and repeatable parsing |
| `decode` | Repeated decoding produces the same value or error; borrowed output stays within the raw value |
| `lookup` | First and all lookups match ordered, ASCII-case-insensitive filtering |
| `python` | Headers, raw values, body, envelope, and defect order match CPython 3.12.13 `compat32` |
| `python_decode` | Decoded ASCII input matches Python's text or error category for the supported codec families |

The Python target keeps one oracle process per fuzzer process. It uses the same
byte-preserving JSON protocol as the [conformance suite](conformance.md), with
no malformed-input exclusions. Set `ASTRAL_EMAIL_PYTHON` to select the pinned
interpreter. The decoder target checks every header parsed directly from the
input, then wraps the input as one value with continuation indentation after
line endings, preserving CRLF pairs.

The `python_decode` target uses the [decoder comparison pipeline](decoding.md#checking-the-decoder)
with a separate persistent Python process. It explores malformed markers,
charset labels, and arbitrary word sequences. It compares successful text and
error categories, including precedence when several words are invalid.
Non-ASCII input is excluded from this comparison because ordinary
Unicode text follows a separate convenience policy. The `decode` invariant target
continues to cover arbitrary bytes.

`fuzz/seed_corpus.py` imports the raw parser and decoder fixtures, captured uv
files, and their header values. Large uv bodies are excluded from the size-limited
corpus, but their header sections still seed it. Inputs are deduplicated by
SHA-256; the report records fixture hashes, counts, and size exclusions. Generated
corpora are not committed.

## Run

Install a nightly Rust toolchain, CPython 3.12.13, and `cargo-fuzz`, then run from
the repository root:

```console
cargo install cargo-fuzz --version 0.13.2 --locked
python3 fuzz/seed_corpus.py
cargo +nightly fuzz run python fuzz/generated/python -- -dict=fuzz/email.dict -max_len=16384 -len_control=0 -max_total_time=900 -timeout=5 -rss_limit_mb=2048 -print_final_stats=1
```

Repeat for each target. `cargo-fuzz` enables AddressSanitizer by default.
Reproduce and minimize a saved failure with:

```console
cargo +nightly fuzz run python fuzz/artifacts/python/crash-HASH
cargo +nightly fuzz tmin python fuzz/artifacts/python/crash-HASH
```

Retain minimized failures as regression tests.

## CI

The [Linux workflow](../.github/workflows/fuzz.yml) runs each target for 30 seconds
on pull requests and pushes, or 15 minutes on scheduled and manual runs. It pins
Rust, CPython, and `cargo-fuzz`, and uses the committed fuzz lockfile. Successful
default-branch campaigns cache inputs for later runs. Logs, compiler and revision
information, input hashes, generated inputs, and failures are retained for
30 days.

Campaign inputs are limited to 16 KiB, with a five-second timeout and a 2 GiB
resident-memory ceiling for the fuzzer. The separate Python oracle has a 512 MiB
address-space ceiling. These are harness limits; the parser has no corresponding
input limit. Short campaigns do not establish sustained coverage, and larger
inputs need separate validation.
