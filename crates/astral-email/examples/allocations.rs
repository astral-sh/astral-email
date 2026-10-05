//! Allocation counts for equivalent parsing and owned field extraction.

#[allow(dead_code)]
#[path = "../tests/support/mod.rs"]
mod support;

use std::alloc::System;
use std::hint::black_box;

use astral_email::Message;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn measure<T>(run: impl FnOnce() -> T) -> Stats {
    let region = Region::new(ALLOCATOR);
    drop(black_box(run()));
    region.change()
}

fn main() {
    println!("case,parser,allocations,reallocations,bytes_allocated,bytes_reallocated");
    for case in support::cases() {
        assert_eq!(
            support::astral_email(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
        let astral = measure(|| support::astral_email(&case.input, case.kind));
        let baseline = measure(|| support::mailparse(&case.input, case.kind));
        for (name, stats) in [("astral-email", astral), ("mailparse", baseline)] {
            println!(
                "{},{},{},{},{},{}",
                case.name,
                name,
                stats.allocations,
                stats.reallocations,
                stats.bytes_allocated,
                stats.bytes_reallocated,
            );
        }
    }

    // Reading an opaque body must not allocate in proportion to its size.
    let headers = b"Name: demo\nVersion: 1\n\n";
    let mut large_body = headers.to_vec();
    large_body.resize(headers.len() + 16 * 1024 * 1024, b'x');
    assert_eq!(
        measure(|| Message::parse(headers)),
        measure(|| Message::parse(&large_body)),
    );
}
