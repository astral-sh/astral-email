#![no_main]

#[path = "../../crates/astral-email/tests/support/conformance.rs"]
#[allow(dead_code)]
mod conformance;

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock};

use libfuzzer_sys::fuzz_target;
use serde_json::{Value, json};

struct Oracle {
    _child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Oracle {
    fn start() -> Self {
        let python = std::env::var_os("ASTRAL_EMAIL_PYTHON").unwrap_or_else(|| "python3".into());
        let mut child = Command::new(python)
            .arg("-u")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("python_oracle.py"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("start CPython 3.12.13 oracle");
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            _child: child,
            input,
            output,
        }
    }

    fn inspect(&mut self, source: &[u8]) -> Value {
        let input_hex: String = source.iter().map(|byte| format!("{byte:02x}")).collect();
        serde_json::to_writer(&mut self.input, &json!({ "input_hex": input_hex })).unwrap();
        writeln!(self.input).unwrap();
        self.input.flush().unwrap();
        let mut response = String::new();
        assert_ne!(
            self.output.read_line(&mut response).unwrap(),
            0,
            "Python oracle exited"
        );
        serde_json::from_str(&response).expect("Python oracle returned JSON")
    }
}

static ORACLE: OnceLock<Mutex<Oracle>> = OnceLock::new();

fuzz_target!(|source: &[u8]| {
    if source.len() > 16_384 {
        return;
    }
    let expected = ORACLE
        .get_or_init(|| Mutex::new(Oracle::start()))
        .lock()
        .unwrap()
        .inspect(source);
    assert_eq!(conformance::inspect(source), expected);
});
