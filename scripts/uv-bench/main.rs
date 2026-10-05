//! Benchmark complete uv metadata parsing, including semantic parsing and drop.

use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use uv_install_wheel::{Error, WheelFile};
use uv_pypi_types::{Metadata23, MetadataError, ResolutionMetadata};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Clone, Copy)]
enum Kind {
    Resolution,
    PkgInfo,
    Publish,
    Wheel,
}

struct Case {
    name: String,
    input: Vec<u8>,
    kind: Kind,
}

/// Keep the actual semantic outputs alive through the timed call and its drop.
enum Output {
    Resolution(Result<ResolutionMetadata, MetadataError>),
    Publish(Result<Metadata23, MetadataError>),
    Wheel(Result<WheelFile, Error>),
}

impl Output {
    fn snapshot(&self) -> Value {
        match self {
            Self::Resolution(Ok(value)) => json!({"ok": value}),
            Self::Publish(Ok(value)) => json!({"ok": value}),
            Self::Wheel(Ok(value)) => json!({"ok": format!("{value:?}")}),
            Self::Resolution(Err(error)) | Self::Publish(Err(error)) => {
                json!({"error": format!("{error:?}")})
            }
            Self::Wheel(Err(error)) => json!({"error": format!("{error:?}")}),
        }
    }
}

impl Case {
    fn run(&self) -> Output {
        let input = black_box(self.input.as_slice());
        match self.kind {
            Kind::Resolution => Output::Resolution(ResolutionMetadata::parse_metadata(input)),
            Kind::PkgInfo => Output::Resolution(ResolutionMetadata::parse_pkg_info(input)),
            Kind::Publish => Output::Publish(Metadata23::parse(input)),
            Kind::Wheel => Output::Wheel(WheelFile::parse(str::from_utf8(input).unwrap())),
        }
    }
}

fn cases(directory: &Path) -> Vec<Case> {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    let mut cases = Vec::new();
    for fixture in manifest["cases"].as_array().unwrap() {
        let file = fixture["file"].as_str().unwrap();
        let input = std::fs::read(directory.join(file)).unwrap();
        let kinds = if fixture["kind"] == "wheel" {
            vec![("wheel", Kind::Wheel)]
        } else {
            let mut kinds = vec![("resolution", Kind::Resolution), ("publish", Kind::Publish)];
            if file.contains("PKG-INFO") {
                kinds.push(("pkg-info", Kind::PkgInfo));
            }
            kinds
        };
        for (suffix, kind) in kinds {
            cases.push(Case {
                name: format!("{}-{suffix}", fixture["name"].as_str().unwrap()),
                input: input.clone(),
                kind,
            });
        }
    }
    for count in [10, 100, 1_000] {
        let mut input = "Metadata-Version: 2.4\nName: demo\nVersion: 1.0\n".to_owned();
        for index in 0..count {
            input.push_str(&format!(
                "Requires-Dist: dependency-{index}>=1.0; python_version >= '3.10'\n"
            ));
        }
        cases.push(Case {
            name: format!("dependencies-{count}"),
            input: input.into_bytes(),
            kind: Kind::Resolution,
        });
    }
    for (name, value) in [
        ("folds", "A package\n with folded\n\tdescription text"),
        ("encoded-words", "=?utf-8?b?5Lit5paH?= =?utf-8?q?_package?="),
    ] {
        let input = format!(
            "Metadata-Version: 2.4\nName: demo\nVersion: 1.0\n{}",
            format!("Classifier: {value}\n").repeat(100)
        );
        cases.push(Case {
            name: name.into(),
            input: input.into_bytes(),
            kind: Kind::Publish,
        });
    }
    cases
}

fn batch(case: &Case, iterations: u64) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(case.run());
    }
    start.elapsed()
}

fn main() {
    let mut directory = None;
    let mut samples = 31;
    let mut sample_ms = 5;
    let mut warmup_ms = 15;
    let mut filter = None;
    let mut snapshot = None;
    let mut check = None;
    let mut json_output = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixtures" => directory = args.next(),
            "--samples" => samples = args.next().unwrap().parse::<usize>().unwrap(),
            "--sample-ms" => sample_ms = args.next().unwrap().parse::<u64>().unwrap(),
            "--warmup-ms" => warmup_ms = args.next().unwrap().parse::<u64>().unwrap(),
            "--filter" => filter = args.next(),
            "--snapshot" => snapshot = args.next(),
            "--check" => check = args.next(),
            "--json" => json_output = true,
            _ => panic!("unknown argument: {arg}"),
        }
    }
    assert!(samples > 0 && sample_ms > 0);
    let cases: Vec<_> = cases(Path::new(&directory.expect("--fixtures is required")))
        .into_iter()
        .filter(|case| {
            filter
                .as_ref()
                .is_none_or(|filter| case.name.contains(filter))
        })
        .collect();
    assert!(!cases.is_empty());
    let outputs: Value = cases
        .iter()
        .map(|case| (case.name.clone(), case.run().snapshot()))
        .collect();
    if let Some(path) = check {
        let expected: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for case in &cases {
            assert_eq!(outputs[&case.name], expected[&case.name], "{}", case.name);
        }
    }
    if let Some(path) = snapshot {
        std::fs::write(path, serde_json::to_vec_pretty(&outputs).unwrap()).unwrap();
        return;
    }
    if !json_output {
        println!("case,bytes,median_ns,p10_ns,p90_ns");
    }
    let mut results = Vec::new();
    for case in cases {
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(warmup_ms) {
            black_box(case.run());
        }
        let mut iterations = 1;
        let count = loop {
            let elapsed = batch(&case, iterations);
            if elapsed >= Duration::from_millis(1) {
                break ((sample_ms as f64 / elapsed.as_secs_f64() / 1_000.0) * iterations as f64)
                    .ceil()
                    .clamp(1.0, 1_000_000.0) as u64;
            }
            iterations *= 2;
        };
        let mut times: Vec<_> = (0..samples)
            .map(|_| batch(&case, count).as_nanos() as f64 / count as f64)
            .collect();
        times.sort_by(f64::total_cmp);
        let p10 = times[(samples - 1) / 10];
        let median = times[(samples - 1) / 2];
        let p90 = times[(samples - 1) * 9 / 10];
        if json_output {
            results.push(json!({"case":case.name,"bytes":case.input.len(),"median_ns":median,"p10_ns":p10,"p90_ns":p90}));
        } else {
            println!(
                "{},{},{median:.0},{p10:.0},{p90:.0}",
                case.name,
                case.input.len()
            );
        }
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    }
}
