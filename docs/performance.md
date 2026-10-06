# Benchmarks

Every performance benchmark uses unmodified files from published Python packages.
The [corpus](../crates/astral-mail-headers/tests/fixtures/packages/manifest.json)
contains nine wheel `METADATA` files, their nine `WHEEL` files, and requests'
sdist `PKG-INFO`. Synthetic inputs remain correctness tests only.

| Package | METADATA bytes | Header bytes | Fields | Requires-Dist |
| --- | ---: | ---: | ---: | ---: |
| backcall 0.2.0 | 1,966 | 478 | 13 | 0 |
| six 1.16.0 | 1,795 | 615 | 17 | 0 |
| packaging 26.3 | 3,544 | 1,288 | 29 | 0 |
| requests 2.32.3 | 4,610 | 1,681 | 40 | 6 |
| Black 24.8.0 | 78,181 | 2,088 | 46 | 13 |
| setuptools 75.1.0 | 6,855 | 4,247 | 84 | 54 |
| NumPy 2.1.2 | 60,936 | 56,958 | 36 | 0 |
| pandas 2.2.3 | 89,901 | 78,364 | 134 | 85 |
| Apache Airflow 2.10.2 | 44,670 | 33,807 | 676 | 486 |

Header bytes include the blank separator before the description. WHEEL files
range from 82 to 110 bytes. NumPy and pandas contain long folded license headers;
Black has a large description; packaging uses SPDX license metadata. NumPy and
pandas use macOS x86_64 wheels, whose bundled license text is platform-specific.

The manifest pins archive URLs, SHA-256 hashes, archive members, fixture hashes,
and Python's expected headers. Included license and notice files retain the
packages' attribution. Regenerate from the pinned archives, or check locally
without network access:

```console
python3 scripts/update_package_fixtures.py
python3 scripts/update_package_fixtures.py --check
```

## Workloads

| Benchmark label | Work measured |
| --- | --- |
| `raw-header-parsing` | Collect borrowed header names and undecoded values, leaving the body opaque. |
| `dependency-metadata-extraction` | Parse and decode `Name`, `Version`, `Requires-Dist`, `Requires-Python`, `Provides-Extra`, and `Dynamic`. |
| `publication-metadata-extraction` | Parse and decode publication fields, including `Author`, `License`, and `Classifier`, and extract the description. |
| `wheel-metadata-extraction` | Parse and decode all WHEEL headers. |

Extraction produces owned field values; it does not solve dependencies, build
packages, or upload them. Fixture loading is excluded from measurements;
output destruction is included.

## Wall-clock timing and allocations

The `metadata-extraction` benchmark compares all 29 extraction workloads against
mailparse, checking both parsers' output before timing.

```console
cargo bench --locked --bench metadata-extraction
cargo bench --locked --bench metadata-extraction --features benchmark-jemalloc
cargo run --release --locked --example allocations
```

The timing benchmark prints CSV with total bytes, header bytes, and median and
p10/p90 timings. Use `-- --filter dependency-metadata-extraction` to select a workload
type, or filter by package name. `-- --test` checks outputs without timing.
Allocation counts and requested bytes use the system allocator.

## README results

The README tables show the median of three session medians, converted from
nanoseconds to microseconds and rounded to two decimal places. Each session ran
all 29 extraction workloads with 31 samples per parser, a 20 ms sample target,
and a 100 ms combined warmup per case. Parser order alternates between samples;
case order is fixed. Each session checks that both parsers produce equal output
before timing. Input loading is excluded; output destruction is included.

The five displayed packages cover small headers (backcall and requests), many
dependencies (Apache Airflow), and long folded license headers (NumPy and pandas).
The dependency and publication tables use wheel `METADATA`; the WHEEL table uses
the corresponding `WHEEL` files. The complete results, including the other four
packages and requests' sdist `PKG-INFO`, are preserved in
[session 1](../benchmarks/results/epyc-vm-e4fd8cb-session-1.csv),
[session 2](../benchmarks/results/epyc-vm-e4fd8cb-session-2.csv), and
[session 3](../benchmarks/results/epyc-vm-e4fd8cb-session-3.csv).
CSV timings are in nanoseconds and retain each session's median and p10/p90.

Measurements were collected on October 6, 2026, from clean source at
[e4fd8cb](https://github.com/astral-sh/astral-mail-headers/tree/e4fd8cb13122da7ef08cd750fb944b77fe76c524),
using mailparse 0.16.1, the system allocator, thin LTO, and one codegen unit.
The compiler was `rustc 1.98.1-dev (f62703110 2026-09-08)` from `ohm-1.98.1-1`,
with Ohm's experimental Cargo defaults disabled. The shared KVM VM used an AMD
EPYC-Milan processor, Linux 6.8.0, and glibc 2.39. Runs were pinned to guest vCPU 0;
physical-core exclusivity, host scheduling, CPU frequency, and neighboring
workloads were not controlled. These measurements describe this corpus and host.
The [run metadata](../benchmarks/results/epyc-vm-e4fd8cb.json) records commands,
timestamps, compiler details, and dependency, fixture-manifest, and binary hashes.

To reproduce the build and three sessions, use the benchmark executable printed
by Cargo and run the second command three times:

```console
cargo +ohm -Zohm-defaults=no bench --locked --bench metadata-extraction --no-run
taskset -c 0 <benchmark-executable> --bench --samples 31 --sample-ms 20 --warmup-ms 100
```

## CodSpeed

CodSpeed tracks instructions and allocations for the same 29 extraction workloads
and raw header parsing of all 19 files. Raw parsing throughput counts header bytes;
extraction throughput counts complete operations.

```console
cargo install cargo-codspeed --version 5.0.1 --locked
cargo codspeed build -m simulation -m memory --profile profiling -p astral-mail-headers --bench codspeed --locked
cargo codspeed run --bench codspeed
```

Local runs check that the benchmarks execute. [CI](../.github/workflows/benchmarks.yml)
records and uploads measurements.
