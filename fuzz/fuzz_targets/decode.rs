#![no_main]

use std::borrow::Cow;

#[path = "../../crates/astral-email/tests/support/decode_input.rs"]
mod decode_input;

use astral_email::{Header, Message};
use libfuzzer_sys::fuzz_target;

fn check_value(header: &Header<'_>) {
    let first = header.decoded_value();
    assert_eq!(first, header.decoded_value());
    if let Ok(Cow::Borrowed(value)) = first
        && !value.is_empty()
    {
        let raw = header.raw_value();
        let start = (value.as_ptr() as usize)
            .checked_sub(raw.as_ptr() as usize)
            .expect("decoded value borrows its raw header");
        assert!(start <= raw.len());
        assert!(value.len() <= raw.len() - start);
        assert_eq!(value.as_bytes(), &raw[start..start + value.len()]);
    }
}

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() > 16_384 {
        return;
    }
    for header in Message::parse(bytes).headers() {
        check_value(header);
    }
    let source = decode_input::wrap(bytes);
    let message = Message::parse(&source);
    check_value(message.first("X").unwrap());
});
