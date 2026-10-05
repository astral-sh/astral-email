#![no_main]

#[path = "../../crates/astral-email/tests/support/conformance.rs"]
#[allow(dead_code)]
mod conformance;

#[path = "support/python.rs"]
mod python;

use std::sync::{Mutex, OnceLock};

use libfuzzer_sys::fuzz_target;
use python::Oracle;
use serde_json::json;

static ORACLE: OnceLock<Mutex<Oracle>> = OnceLock::new();

fuzz_target!(|source: &[u8]| {
    if source.len() > 16_384 {
        return;
    }
    let input_hex: String = source.iter().map(|byte| format!("{byte:02x}")).collect();
    let expected = ORACLE
        .get_or_init(|| Mutex::new(Oracle::start("headers")))
        .lock()
        .unwrap()
        .inspect(&json!({ "input_hex": input_hex }));
    assert_eq!(conformance::inspect(source), expected);
});
