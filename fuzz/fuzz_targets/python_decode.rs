#![no_main]

#[path = "support/python.rs"]
mod python;

use std::sync::{Mutex, OnceLock};

use astral_email::Message;
use libfuzzer_sys::fuzz_target;
use python::Oracle;
use serde_json::json;

static ORACLE: OnceLock<Mutex<Oracle>> = OnceLock::new();

fuzz_target!(|value: &[u8]| {
    // Ordinary Unicode text has a separate convenience policy.
    if value.len() > 16_384 || !value.is_ascii() {
        return;
    }
    let value = str::from_utf8(value).unwrap();
    let source = format!("X:{value}");
    let message = Message::parse(source.as_bytes());
    let actual = message.first("X").unwrap().decoded_value();
    let expected = ORACLE
        .get_or_init(|| Mutex::new(Oracle::start("decode")))
        .lock()
        .unwrap()
        .inspect(&json!({ "value": value }));
    // Both must reject invalid values; multiple errors have no precedence contract.
    if expected.get("error").is_some() {
        assert!(
            actual.is_err(),
            "Python rejected a value we accepted: {value:?}"
        );
    } else {
        assert_eq!(
            actual.as_deref().ok(),
            Some(expected["value"].as_str().expect("Python returned text")),
            "{value:?}",
        );
    }
});
