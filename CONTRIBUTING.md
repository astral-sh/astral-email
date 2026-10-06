# Contributing

## Checks

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Check Python compatibility with CPython 3.12.13 on a little-endian host:

```console
python3 scripts/generate_conformance.py --check
python3 scripts/generate_decode_fixtures.py --check
cargo build -p astral-mail-headers --example decode_inspect --locked
python3 scripts/check_decode.py target/debug/examples/decode_inspect
```

The decoder differential check covers ASCII inputs and supported codecs.
Omit `--check` to regenerate fixtures. For individual inputs, the generators'
`--stdin` mode and the Rust [inspect examples](crates/astral-mail-headers/examples/)
accept JSON lines: `input_hex` for raw parsing or `value` for decoding.
See [Benchmarks](docs/performance.md) for performance checks.

## uv integration

Check out uv [`46b84fd0bfec23b72f29e8e2185ba68a65052f48`](https://github.com/astral-sh/uv/tree/46b84fd0bfec23b72f29e8e2185ba68a65052f48)
into `uv-source` at the repository root, then run:

```console
python3 scripts/test_uv.py
```

The script applies the [adapter](scripts/uv-tests/uv-integration.patch) to uv's metadata and
WHEEL readers and runs upstream and adapter tests, verifying that upstream test
modules remain unchanged. The adapter accepts parser defects, propagates decoding
errors, and leaves description UTF-8 validation to uv.

Regenerate the [uv fixtures](crates/astral-mail-headers/tests/fixtures/uv/manifest.json)
with `python3 scripts/update_uv_fixtures.py uv-source`. The manifest records source
hashes and Python expectations; licenses are retained alongside the fixtures.

## Fuzzing

| Target | Checks |
| --- | --- |
| `reader` | Borrowed ranges, header order, valid names, envelope position, and repeatable parsing |
| `decode` | Repeatable decoding and borrowed output ranges for arbitrary bytes |
| `lookup` | Ordered, ASCII-case-insensitive first and all lookups |
| `python` | Raw headers, body, envelope, and defects against Python `compat32` |
| `python_decode` | ASCII inputs against Python decoding with supported codecs |

The Python targets require CPython 3.12.13; set `ASTRAL_EMAIL_PYTHON` if it is not
`python3`. Non-ASCII text follows the separate [decoding policy](docs/decoding.md).

Install a nightly Rust toolchain, then run from the repository root:

```console
cargo install cargo-fuzz --version 0.13.2 --locked
python3 fuzz/seed_corpus.py
cargo +nightly fuzz run python fuzz/generated/python -- -dict=fuzz/email.dict -max_len=16384 -len_control=0 -max_total_time=900 -timeout=5 -rss_limit_mb=2048 -print_final_stats=1
```

Replace `python` in the target and corpus path to run another target. Reproduce
and minimize a saved failure with `cargo +nightly fuzz run python PATH` and
`cargo +nightly fuzz tmin python PATH`, then retain it as a regression test.
[CI](.github/workflows/fuzz.yml) runs short PR checks and longer scheduled campaigns
with AddressSanitizer, saving logs, corpora, and failures as artifacts.

## Releases

Releases can only be performed by Astral team members.

### Configuration

Release gates and tag protections are managed in
[github-policies](https://github.com/astral-sh/github-policies). Before the first
release, run **Apply** in [crates-policies](https://github.com/astral-sh/crates-policies)
to bootstrap the crate and enable Trusted Publishing for `release.yml` in the
`release` environment.

The workflows use these environment secrets:

| Environment   | Secret           | Value                                                     |
| ------------- | ---------------- | --------------------------------------------------------- |
| `automations` | `STS_API_URL`    | Astral's automation-broker base URL, without `/exchange`. |
| `automations` | `OPENAI_API_KEY` | An API key for the Codex changelog rewrite.               |
| `release`     | `STS_API_URL`    | Astral's release-broker base URL, without `/exchange`.    |

### Prepare and publish

1. Run **Prepare release** from `main`. Leave `version` empty to use `Cargo.toml`
   for the first release, then automatically bump the minor version for `breaking`
   changes or the patch version otherwise, excluding internal changes. To override,
   provide an exact stable Cargo version without a leading `v`.
2. Review and merge the generated version and changelog PR.
3. Run **Release** from `main` with that version. **Dry-run** validates the package
   and release notes without publishing.
4. Have another Astral team member approve `release-gate` to publish the crate and
   create the GitHub release. Retries skip crate uploads already on crates.io.
