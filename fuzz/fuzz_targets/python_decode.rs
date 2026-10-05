#![no_main]

#[path = "support/python.rs"]
mod python;

use std::sync::{Mutex, OnceLock};

use astral_mail_headers::{DecodeError, Message};
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
    let actual = match message.first("X").unwrap().decoded_value() {
        Ok(value) => json!({ "value": value }),
        Err(DecodeError::InvalidBase64) => json!({ "error": "invalid_base64" }),
        Err(DecodeError::UnsupportedCharset(_)) => json!({ "error": "unsupported_charset" }),
    };
    let expected = ORACLE
        .get_or_init(|| Mutex::new(Oracle::start("decode")))
        .lock()
        .unwrap()
        .inspect(&json!({ "value": value }));
    assert_eq!(actual, expected, "{value:?}");
});
