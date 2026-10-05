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
    let mut args = std::env::args().skip(1);
    let reuse = match args.next().as_deref() {
        None => false,
        Some("--reuse") => true,
        Some(arg) => panic!("unknown argument: {arg}"),
    };
    assert!(args.next().is_none());
    let mut parser = astral_email::Parser::default();
    println!("case,parser,allocations,reallocations,bytes_allocated,bytes_reallocated");
    for case in support::cases() {
        assert_eq!(
            support::astral_email(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
        let results = if reuse {
            assert_eq!(
                support::astral_email(&case.input, case.kind),
                support::astral_email_reused(&mut parser, &case.input, case.kind),
            );
            [
                (
                    "astral-reused",
                    measure(|| support::astral_email_reused(&mut parser, &case.input, case.kind)),
                ),
                (
                    "astral-fresh",
                    measure(|| support::astral_email(&case.input, case.kind)),
                ),
            ]
        } else {
            [
                (
                    "astral-email",
                    measure(|| support::astral_email(&case.input, case.kind)),
                ),
                (
                    "mailparse",
                    measure(|| support::mailparse(&case.input, case.kind)),
                ),
            ]
        };
        for (name, stats) in results {
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
