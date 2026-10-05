# Fuzzing

Four targets accept arbitrary bytes:

| Target | Checks |
| --- | --- |
| `reader` | Borrowed field ranges, header order, valid names, envelope position, and repeatable parsing |
| `decode` | Repeated decoding produces the same value or error; borrowed output stays within the raw value |
| `lookup` | First and all lookups match ordered, ASCII-case-insensitive filtering |
| `python` | Headers, raw values, body, envelope, and defect order match CPython 3.12.13 `compat32` |

The Python target keeps one oracle process per fuzzer process. It uses the same
byte-preserving JSON protocol as the [conformance suite](conformance.md), with
no malformed-input exclusions. Set `ASTRAL_EMAIL_PYTHON` to select the pinned
interpreter. The decoder target inserts continuation indentation after line
endings so every input byte can reach a header value. It checks invariants;
it is not an independent oracle for decoded values.

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
