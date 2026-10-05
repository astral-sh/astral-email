//! Regression coverage for the decoder fuzz input wrapper.

#[path = "support/decode_input.rs"]
mod decode_input;

use astral_email::Message;

#[test]
fn wraps_values_without_splitting_line_endings() {
    for (input, expected) in [
        (b"plain".as_slice(), b"plain".as_slice()),
        (b"first\r\nSecond: value", b"first\r\n Second: value"),
        (b"first\rSecond: value", b"first\r Second: value"),
        (b"first\nSecond: value", b"first\n Second: value"),
    ] {
        let source = decode_input::wrap(input);
        let message = Message::parse(&source);
        assert_eq!(message.headers().len(), 1);
        assert_eq!(message.first("X").unwrap().raw_value(), expected);
    }
}
