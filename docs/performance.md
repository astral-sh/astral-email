# Benchmarks

The `parse` benchmark compares parsing and owned field extraction against mailparse
using [uv fixtures](uv.md#fixtures) and synthetic inputs. It checks that both
parsers produce the same output before timing. Fixture loading is excluded;
output destruction is included.

```console
cargo bench --locked --bench parse
cargo bench --locked --bench parse --features benchmark-jemalloc
```

The benchmark prints CSV with median and p10/p90 timings. Use `-- --filter NAME`
to select workloads or `-- --test` to check outputs without timing. For allocation
counts and requested bytes with the system allocator, run:

```console
cargo run --release --locked --example allocations
```

## CodSpeed

CodSpeed tracks instructions and allocations for parsing, metadata extraction,
and header decoding. Fixture setup and, for decoding benchmarks, message parsing
are excluded; output destruction is included.

```console
cargo install cargo-codspeed --version 5.0.1 --locked
cargo codspeed build -m simulation -m memory --profile profiling -p astral-mail-headers --bench codspeed --locked
cargo codspeed run --bench codspeed
```

Local runs check that the benchmarks execute. [CI](../.github/workflows/benchmarks.yml)
records and uploads measurements.
