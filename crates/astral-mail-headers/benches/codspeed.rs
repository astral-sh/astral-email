//! Raw header parsing and owned metadata extraction from published Python packages.

#![allow(missing_docs, reason = "Criterion macros generate public functions")]

#[allow(dead_code)]
#[path = "../tests/support/mod.rs"]
mod support;

use std::hint::black_box;

use astral_mail_headers::Message;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn extract(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata-extraction");
    for case in support::benchmark_cases() {
        assert_eq!(
            support::astral_mail_headers(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
        group.throughput(Throughput::Elements(1));
        group.bench_function(&case.name, |b| {
            b.iter(|| {
                black_box(support::astral_mail_headers(
                    black_box(&case.input),
                    case.kind,
                ));
            });
        });
    }
    group.finish();
}

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("raw-header-parsing");
    for (fixture, input) in support::benchmark_fixtures() {
        group.throughput(Throughput::Bytes(fixture.body_start as u64));
        group.bench_function(&fixture.name, |b| {
            b.iter(|| {
                black_box(Message::parse(black_box(&input)));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, extract, parse);
criterion_main!(benches);
