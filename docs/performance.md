# Performance

The [81 uv workloads](uv.md#fixtures-and-benchmarks) run faster than mailparse
0.16.1 in both repetitions with both allocators. These measurements include
parsing, equivalent owned field extraction, and output destruction.

| Allocator | Geometric mean speedup, run 1 | Run 2 | Smallest speedup across both runs |
| --- | ---: | ---: | ---: |
| System | 2.39× | 2.37× | 1.78× |
| jemalloc | 2.25× | 2.25× | 1.74× |

The [per-workload results](../benchmarks/results.csv) retain medians, p10/p90,
and allocation counts. [Environment and source hashes](../benchmarks/environment.json)
identify the measured implementation. Ratios above are mailparse time divided by
astral-email time; the geometric mean weights each workload equally.

Measurements were taken on 2026-10-05 on a Linux x86-64 AMD EPYC Milan virtual
machine, pinned to CPU 26. Both parsers used the same release binary and allocator,
with thin LTO and one codegen unit. The compiler was Rust 1.98.1-dev, commit
`f6270311094cd4b48fefce03debdffcf8396c64c`, LLVM 22.1.8, with experimental build
defaults disabled and no additional compiler flags. The binaries were rebuilt
from the recorded source hashes before timing. No local builds or fuzzing ran
during measurement.

Each workload warmed up for 15 ms, then took 31 samples of approximately 5 ms per
parser, alternating parser order. The four runs used system, jemalloc, jemalloc,
then system, to check repeatability in reverse allocator order. These results
describe this corpus and machine; they do not measure whole uv commands or
establish performance on ARM64. ARM64 correctness is checked in CI.

## Reproduce

Run on an otherwise idle machine, choosing an available CPU:

```console
taskset -c 26 cargo bench --locked --bench parse -- --samples 31 --sample-ms 5 --warmup-ms 15
taskset -c 26 cargo bench --locked --bench parse --features benchmark-jemalloc -- --samples 31 --sample-ms 5 --warmup-ms 15
```

Repeat in reverse order. Fixture loading and output comparisons happen before
timing. The benchmark prints CSV; `--test` checks the outputs without timing.

## Allocations

```console
cargo run --release --locked --example allocations
```

This runs the same extraction through `stats_alloc` around the system allocator,
with fixture loading outside the measured region. Every workload makes fewer
allocation or reallocation calls: reductions range from 33% to 75%. Requested
allocation bytes fall by 17% to 86%. These are allocator requests, not peak RSS;
reallocation growth is included in `bytes_allocated` and reported separately in
`bytes_reallocated`.

The example also checks that parsing an ordinary opaque 16 MiB body allocates
exactly as much as parsing identical headers with an empty body. Python's unusual
trailing-envelope recovery can require copying the body, as described in the
[conformance contract](conformance.md#raw-parsing).
