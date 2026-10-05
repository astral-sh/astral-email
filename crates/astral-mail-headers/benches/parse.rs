//! Parse and extract owned packaging fields with both parsers.

#[allow(dead_code)]
#[path = "../tests/support/mod.rs"]
mod support;

use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(all(
    feature = "benchmark-jemalloc",
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

struct Options {
    samples: usize,
    sample: Duration,
    warmup: Duration,
    filter: Option<String>,
    test: bool,
}

impl Options {
    fn parse() -> Self {
        let mut options = Self {
            samples: 31,
            sample: Duration::from_millis(5),
            warmup: Duration::from_millis(10),
            filter: None,
            test: cfg!(debug_assertions),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--test" => options.test = true,
                "--bench" => {}
                "--samples" => options.samples = args.next().unwrap().parse().unwrap(),
                "--sample-ms" => {
                    options.sample = Duration::from_millis(args.next().unwrap().parse().unwrap());
                }
                "--warmup-ms" => {
                    options.warmup = Duration::from_millis(args.next().unwrap().parse().unwrap());
                }
                "--filter" => options.filter = Some(args.next().unwrap()),
                _ => panic!("unknown argument: {arg}"),
            }
        }
        assert!(options.samples > 0 && !options.sample.is_zero());
        options
    }
}

fn batch(run: &impl Fn() -> support::Output, iterations: u64) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(run());
    }
    start.elapsed()
}

fn iterations(run: &impl Fn() -> support::Output, target: Duration) -> u64 {
    let mut count = 1;
    loop {
        let elapsed = batch(run, count);
        if elapsed >= Duration::from_millis(1) {
            return ((target.as_secs_f64() / elapsed.as_secs_f64()) * count as f64)
                .ceil()
                .clamp(1.0, 1_000_000.0) as u64;
        }
        count *= 2;
    }
}

fn percentile(samples: &[f64], percentile: usize) -> f64 {
    samples[(samples.len() - 1) * percentile / 100]
}

fn main() {
    let options = Options::parse();
    let cases: Vec<_> = support::cases()
        .into_iter()
        .filter(|case| {
            options
                .filter
                .as_ref()
                .is_none_or(|filter| case.name.contains(filter))
        })
        .collect();
    assert!(!cases.is_empty(), "no cases matched the filter");
    for case in &cases {
        assert_eq!(
            support::astral_mail_headers(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
    }
    if options.test {
        println!("{} extraction comparisons passed", cases.len());
        return;
    }

    println!("# Parse and extract owned packaging fields; drop included");
    println!(
        "# allocator={} samples={} sample_ms={} warmup_ms={}",
        if cfg!(all(
            feature = "benchmark-jemalloc",
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )) {
            "jemalloc"
        } else {
            "system"
        },
        options.samples,
        options.sample.as_millis(),
        options.warmup.as_millis(),
    );
    println!(
        "case,bytes,astral_mail_headers_ns,mailparse_ns,speedup,astral_mail_headers_p10_ns,astral_mail_headers_p90_ns,mailparse_p10_ns,mailparse_p90_ns"
    );
    for case in cases {
        let astral = || support::astral_mail_headers(black_box(&case.input), case.kind);
        let baseline = || support::mailparse(black_box(&case.input), case.kind);
        let start = Instant::now();
        while start.elapsed() < options.warmup {
            black_box(astral());
            black_box(baseline());
        }
        let astral_iterations = iterations(&astral, options.sample);
        let baseline_iterations = iterations(&baseline, options.sample);
        let mut astral_samples = Vec::with_capacity(options.samples);
        let mut baseline_samples = Vec::with_capacity(options.samples);
        for sample in 0..options.samples {
            let (astral_time, baseline_time) = if sample % 2 == 0 {
                (
                    batch(&astral, astral_iterations),
                    batch(&baseline, baseline_iterations),
                )
            } else {
                let baseline_time = batch(&baseline, baseline_iterations);
                (batch(&astral, astral_iterations), baseline_time)
            };
            astral_samples.push(astral_time.as_nanos() as f64 / astral_iterations as f64);
            baseline_samples.push(baseline_time.as_nanos() as f64 / baseline_iterations as f64);
        }
        astral_samples.sort_by(f64::total_cmp);
        baseline_samples.sort_by(f64::total_cmp);
        let astral_median = percentile(&astral_samples, 50);
        let baseline_median = percentile(&baseline_samples, 50);
        println!(
            "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            case.name,
            case.input.len(),
            astral_median,
            baseline_median,
            baseline_median / astral_median,
            percentile(&astral_samples, 10),
            percentile(&astral_samples, 90),
            percentile(&baseline_samples, 10),
            percentile(&baseline_samples, 90),
        );
    }
}
