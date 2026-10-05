//! Parsing, metadata extraction, and header decoding benchmarks.

#![allow(missing_docs, reason = "Criterion macros generate public functions")]

#[allow(dead_code)]
#[path = "../tests/support/mod.rs"]
mod support;

use std::hint::black_box;

use astral_email::Message;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn extract(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse-and-extract");
    for case in support::cases() {
        assert_eq!(
            support::astral_email(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
        group.throughput(Throughput::Bytes(case.input.len() as u64));
        group.bench_function(&case.name, |b| {
            b.iter(|| {
                black_box(support::astral_email(black_box(&case.input), case.kind));
            });
        });
    }
    group.finish();
}

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    for (fixture, input) in support::fixtures() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(&fixture.name, |b| {
            b.iter(|| {
                black_box(Message::parse(black_box(&input)));
            });
        });
    }
    group.finish();
}

fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    for case in support::cases().into_iter().filter(|case| {
        case.name.starts_with("synthetic-")
            || matches!(case.name.as_str(), "folds" | "encoded-words")
    }) {
        let message = Message::parse(&case.input);
        let Some(header) = message.first("Classifier") else {
            continue;
        };
        group.throughput(Throughput::Bytes(header.raw_value().len() as u64));
        group.bench_function(&case.name, |b| {
            b.iter(|| {
                black_box(black_box(header).decoded_value().unwrap());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, extract, parse, decode);
criterion_main!(benches);
