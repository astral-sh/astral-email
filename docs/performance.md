# Benchmarks

Every performance benchmark uses unmodified files from published Python packages.
The [corpus](../crates/astral-mail-headers/tests/fixtures/packages/manifest.json)
contains nine wheel `METADATA` files, their nine `WHEEL` files, and requests'
sdist `PKG-INFO`. Synthetic inputs and uv's handcrafted packages remain correctness
tests only.

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

## Wall-clock timing and allocations

The `parse` benchmark compares owned field extraction against mailparse: resolution
and publishing for all ten metadata files, plus the nine WHEEL files. It checks
both parsers' output before timing. Fixture loading is excluded; output destruction
is included.

```console
cargo bench --locked --bench parse
cargo bench --locked --bench parse --features benchmark-jemalloc
cargo run --release --locked --example allocations
```

The timing benchmark prints CSV with total bytes, header bytes, and median and
p10/p90 timings. Use `-- --filter NAME` to select workloads or `-- --test` to check
outputs without timing. Allocation counts and requested bytes use the system
allocator. Compare optimization candidates separately for each workload and both
allocators; repeated runs help distinguish changes from timing noise.

## CodSpeed

CodSpeed tracks instructions and allocations for the same 29 extraction workloads
and raw parsing of all 19 files. Parse throughput counts only header bytes, since
the description remains opaque. Extraction throughput counts complete operations.
Fixture setup is excluded; output destruction is included.

```console
cargo install cargo-codspeed --version 5.0.1 --locked
cargo codspeed build -m simulation -m memory --profile profiling -p astral-mail-headers --bench codspeed --locked
cargo codspeed run --bench codspeed
```

Local runs check that the benchmarks execute. [CI](../.github/workflows/benchmarks.yml)
records and uploads measurements.
