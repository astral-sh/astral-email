# Replacing encoding_rs

This compares baseline `b572cc2` with candidate `088ceb6` on October 6, 2026.
The candidate replaces the Latin-1, Windows-1252, and UTF-16 decoders with local
code and standard-library Unicode conversion.

## Published-package corpus

The 29 existing metadata-extraction workloads use unmodified published-package
files. None of the 19 input files contains an RFC 2047 encoded word, so this
comparison checks common-path regressions rather than the changed charset code.
Every session verifies that both parsers produce equal output before timing.

The system allocator measured a 2.8% aggregate slowdown, exceeding the unchanged
baseline's 0.1% aggregate drift. The jemalloc aggregate improved by 1.2%,
alongside 0.4% improvement in its unchanged-baseline control. Results therefore
include a small system-allocator regression.

| Allocator | Candidate aggregate time change |    Per-case range | Unchanged-baseline aggregate drift | Control per-case range |
| --------- | ------------------------------: | ----------------: | ---------------------------------: | ---------------------: |
| System    |                          +2.83% | −12.60% to +6.55% |                             +0.10% |       −2.91% to +3.07% |
| jemalloc  |                          −1.21% | −10.42% to +4.49% |                             −0.43% |       −1.80% to +0.60% |

Aggregates are unweighted geometric means of the 29 per-case timing ratios. Each
candidate ratio compares medians of three session medians. The control reruns
the same baseline executable once after the paired sessions, comparing its
median with the original three-session baseline median. These controls measure
drift; they do not isolate effects of changed binary layout. The median
individual case change was +3.64% for System and +0.33% for jemalloc.

The [summary CSV](corpus-summary.csv) retains every case's absolute timings,
relative change, baseline repeat range, and control change.
[Run metadata](corpus-runs.json) links all 14 raw sessions to their executable
and CSV hashes.

## Supplemental charset measurements

These are **synthetic decoder measurements**, separate from the
published-package benchmark corpus. The identical
[temporary harness](codec-timing.rs) calls the public `Header::decoded_value`
API on [15 inputs](codec-inputs.json). Parsing and input setup are excluded; RFC
2047 transfer decoding, charset conversion, and output destruction are included.
Every input is checked against its Python-decoded expected value before timing.

The short cases use one author-like string; the long cases join 64 copies with
`/`. Their Base64 payloads are split into 40-character chunks and adjacent
encoded words; malformed cases use one encoded word. All words stay below 75
characters. Malformed inputs include undefined Windows-1252 bytes, unpaired
UTF-16 surrogates, and truncated code units. The sample mix is artificial and is
not an estimate of production traffic.

The following changes compare medians of three session medians. Negative values
mean less time. Absolute timings, baseline repeat ranges, and complete per-case
values are in the [summary CSV](supplemental-summary.csv); each session's median
and p10/p90 remain in the raw CSVs under [results](results).

| Input                    | System time change | jemalloc time change |
| ------------------------ | -----------------: | -------------------: |
| `latin1-ascii-short`     |              +0.7% |                -3.9% |
| `latin1-ascii-long`      |              +0.6% |                -1.8% |
| `latin1-non-ascii-short` |              -0.5% |                -3.5% |
| `latin1-non-ascii-long`  |              -1.0% |                -3.3% |
| `cp1252-special-short`   |              -9.2% |               -10.7% |
| `cp1252-special-long`    |              -8.5% |               -11.9% |
| `utf16-native-bom-short` |              -0.3% |                -4.9% |
| `utf16-native-bom-long`  |              +2.1% |                +4.0% |
| `utf16-le-short`         |              -4.4% |                -5.9% |
| `utf16-le-long`          |              +1.4% |                +3.2% |
| `utf16-be-short`         |              -3.3% |                -5.8% |
| `utf16-be-long`          |              -0.0% |                +4.1% |
| `cp1252-invalid`         |             -23.2% |               -27.2% |
| `utf16-le-malformed`     |             -10.5% |               -10.4% |
| `utf16-be-malformed`     |             -10.5% |               -11.0% |

The valid long UTF-16 cases retain a small slowdown: up to 2.1% with the system
allocator and 4.1% with jemalloc. Windows-1252 improves by 8.5–11.9% for the
special-character cases and 23.2–27.2% for undefined bytes; malformed UTF-16
improves by 10.4–11.0%. These observations apply to the listed inputs and this
host.

## Method and reproduction

Both variants use the same compiler, thin LTO, one codegen unit, and pinned
baseline dependency versions. Each session uses 31 samples, a 20 ms sample
target, and 100 ms warmup per case, with output destruction included. Runs are
pinned to guest CPU 0 on the same AMD EPYC-Milan KVM VM. Baseline/candidate
order alternates between sessions, and both system and jemalloc allocators are
measured. Package sessions also alternate astral-mail-headers/mailparse order
within samples. One additional unchanged-baseline package session per allocator
checks drift after the paired runs.

The host is shared: physical-core exclusivity, host scheduling, CPU frequency,
and neighboring workloads are not controlled. Small changes should be read
alongside baseline repeat variability and the unchanged-binary control rather
than treated as statistically established wins or regressions. No confidence
intervals or significance claims are made.

[Environment metadata](environment.json) records compiler, CPU, kernel, libc,
flags, input hashes, and implementation hashes.
[Baseline builds](baseline-build.json) and
[candidate builds](candidate-build.json) record commands and executable hashes.
The [corpus](corpus-runs.json) and [supplemental](supplemental-runs.json) run
manifests record exact commands, UTC timestamps, and binary/CSV hashes. The
measured candidate source is `088ceb6`; later commits only add this evidence and
its documentation.

To reproduce, keep separate worktrees at the two revisions and copy this
directory to a scratch location. The scripts create only an example temporarily
in each worktree, validate outputs, and preserve separate copies of each
executable. The worktrees must remain available because the package benchmark
loads its fixtures from their original paths.

```console
python3 build.py /path/to/baseline baseline
python3 build.py /path/to/candidate candidate
python3 run.py supplemental
python3 run.py corpus
python3 summarize.py
```

Builds use `cargo +ohm -Zohm-defaults=no`; each worktree has a separate `target`
directory, and `CARGO_BUILD_BUILD_DIR` defaults to `~/.cache/ohm-build`. The
scripts require Linux `taskset` and use CPU 0. The supplemental example is an
experiment artifact and is not added to the package's permanent benchmark
targets.
