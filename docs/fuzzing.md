# Fuzzing

| Target          | Checks                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------- |
| `reader`        | Borrowed field ranges, header order, valid names, envelope position, and repeatable parsing |
| `decode`        | Repeatable decoding; borrowed output stays within the raw value                             |
| `lookup`        | First and all lookups match ordered, ASCII-case-insensitive filtering                       |
| `python`        | Headers, raw values, body, envelope, and defect order match Python `compat32`               |
| `python_decode` | Decoded ASCII input matches Python's text or error category for supported codecs            |

The Python targets require CPython 3.12.13. Set `ASTRAL_EMAIL_PYTHON` if it is not
available as `python3`. The `python_decode` target excludes non-ASCII input because
ordinary Unicode follows a separate [decoding policy](decoding.md); `decode`
covers arbitrary bytes.

## Run

Install a nightly Rust toolchain, then run from the repository root:

```console
cargo install cargo-fuzz --version 0.13.2 --locked
python3 fuzz/seed_corpus.py
cargo +nightly fuzz run python fuzz/generated/python -- -dict=fuzz/email.dict -max_len=16384 -len_control=0 -max_total_time=900 -timeout=5 -rss_limit_mb=2048 -print_final_stats=1
```

The seed script imports parser, decoder, and uv fixtures. To run another target,
replace `python` in both the target name and corpus path. `cargo-fuzz` enables
AddressSanitizer by default.

Reproduce and minimize a saved failure:

```console
cargo +nightly fuzz run python fuzz/artifacts/python/crash-HASH
cargo +nightly fuzz tmin python fuzz/artifacts/python/crash-HASH
```

Retain minimized failures as regression tests.

The [CI workflow](../.github/workflows/fuzz.yml) runs short checks on pull requests
and longer scheduled campaigns, retaining logs, corpora, and failures as artifacts.
