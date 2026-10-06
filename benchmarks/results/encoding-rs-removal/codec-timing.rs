//! Supplemental synthetic timing of public RFC 2047 value decoding.
use std::hint::black_box;
use std::time::{Duration, Instant};
use astral_mail_headers::Message;

#[cfg(all(feature = "benchmark-jemalloc", target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")))]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn batch(run: &impl Fn(), count: u64) -> Duration {
    let start = Instant::now();
    for _ in 0..count { run(); }
    start.elapsed()
}

fn main() {
    let path = std::env::args().nth(1).expect("pass input JSON path");
    let cases: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let test = std::env::args().any(|arg| arg == "--test");
    let sample = Duration::from_millis(20);
    println!("# Supplemental synthetic RFC2047 decoded_value; parse/setup excluded, output destruction included");
    println!("# samples=31 sample_ms=20 warmup_ms=100");
    println!("case,input_bytes,output_bytes,median_ns,p10_ns,p90_ns");
    for case in cases.as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let input = case["input"].as_str().unwrap();
        let expected = case["expected"].as_str().unwrap();
        let message = Message::parse(input.as_bytes());
        let header = message.first("X").unwrap();
        assert_eq!(header.decoded_value().unwrap(), expected, "{name}");
        if test { continue; }
        let run = || { black_box(black_box(&header).decoded_value().unwrap()); };
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(100) { run(); }
        let mut count = 1;
        let count = loop {
            let elapsed = batch(&run, count);
            if elapsed >= Duration::from_millis(1) {
                break ((sample.as_secs_f64() / elapsed.as_secs_f64()) * count as f64).ceil().clamp(1.0, 1_000_000.0) as u64;
            }
            count *= 2;
        };
        let mut samples: Vec<_> = (0..31).map(|_| batch(&run, count).as_nanos() as f64 / count as f64).collect();
        samples.sort_by(f64::total_cmp);
        println!("{name},{},{},{:.3},{:.3},{:.3}", input.len(), expected.len(), samples[15], samples[3], samples[27]);
    }
}
