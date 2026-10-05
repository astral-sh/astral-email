#![no_main]

use std::borrow::Cow;

use astral_email::Message;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() > 16_384 {
        return;
    }
    // Keep every input line in one header value.
    let mut source = Vec::with_capacity(bytes.len() * 2 + 5);
    source.extend_from_slice(b"X: ");
    for &byte in bytes {
        source.push(byte);
        if matches!(byte, b'\r' | b'\n') {
            source.push(b' ');
        }
    }
    source.extend_from_slice(b"\n\n");
    let message = Message::parse(&source);
    let header = message.first("X").unwrap();
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
});
