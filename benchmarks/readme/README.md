# README benchmarks

The README tables report the median of three session medians, converted from
nanoseconds to microseconds and rounded to two decimal places. Both parsers use
the system allocator. [Results](results.csv) contain all 24 measurements,
including p10 and p90; [environment and source hashes](environment.json) identify
the clean measured revision and build settings.

## Workloads

The tables use the existing maturin 1.4.0, tqdm 4.66.1, and Black 24.8.0 fixtures,
selected to cover recognizable packages and metadata sizes rather than speedup.
The fixtures were extracted from
[uv's test corpus](https://github.com/astral-sh/uv/tree/46b84fd0bfec23b72f29e8e2185ba68a65052f48/test).
They are test archives, not newly downloaded releases. The
[fixture manifest](../../crates/astral-email/tests/fixtures/uv/manifest.json)
records their upstream revision, archive members, content hashes, and Python's
expected raw headers. All 48 corpus file hashes were verified before measurement.

| Package       | Metadata bytes | WHEEL bytes |
| ------------- | -------------: | ----------: |
| maturin 1.4.0 |          1,177 |         147 |
| tqdm 4.66.1   |          3,633 |          92 |
| Black 24.8.0  |         78,181 |           — |

Resolution extracts owned fields used by uv's resolver. Publishing extracts its
full metadata fields and description. Wheel installation collects owned header
names and decoded values in a map, preserving repeated values. Every timing
includes parsing and output destruction, and both parsers must produce identical
outputs before the timer starts. File loading and output comparisons are excluded.
Black's PKG-INFO includes a long description; its headers and separator occupy
2,088 bytes. There is no Black WHEEL fixture in this corpus.

These are eight extraction workloads, not complete resolution, publishing, or
installation commands. They exclude network, archive, and filesystem work.
The broader benchmark corpus also contains uv test packages and generated
stress cases; those remain available through the benchmark harness.

## Measurement

Measured on 2026-10-05 (America/New_York) on a shared Linux x86-64 AMD EPYC-Milan
VM, pinned to CPU 26. Local builds were paused during measurement. Each workload
warmed up for 15 ms and took 31 samples of approximately 5 ms per parser,
alternating parser order. Three sessions ran all eight cases in package order:
maturin, tqdm, then Black. These results describe this corpus and machine; they
do not establish performance on ARM64.

The baseline is mailparse 0.16.1 with default features disabled. The compiler was
Rust 1.98.1-dev (`f6270311094cd4b48fefce03debdffcf8396c64c`), LLVM 22.1.8, with
Ohm experimental build defaults disabled, thin LTO, one codegen unit, and no
additional compiler flags. Both parsers ran in the same release binary.

## Reproduce

Check out the recorded source revision and choose an available CPU on an
otherwise idle machine:

```console
git checkout 03a2973ea2fa42d0c160009c60c3b16a008b2d44
cargo bench --locked --bench parse --no-run
```

Run the following three times, preserving each run's CSV output:

```console
taskset -c 26 cargo bench --locked --bench parse -- --filter maturin-1.4.0 --samples 31 --sample-ms 5 --warmup-ms 15
taskset -c 26 cargo bench --locked --bench parse -- --filter tqdm-4.66.1 --samples 31 --sample-ms 5 --warmup-ms 15
taskset -c 26 cargo bench --locked --bench parse -- --filter black-24.8.0 --samples 31 --sample-ms 5 --warmup-ms 15
```

For each parser and case, take the median of the three `astral_email_ns` or
`mailparse_ns` values and divide by 1,000. Do not pool samples or average speedup
ratios. This run used `cargo +ohm -Zohm-defaults=no` to disable the local
experimental compiler defaults; the exact compiler version is recorded above.
