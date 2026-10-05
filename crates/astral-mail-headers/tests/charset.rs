//! Charset labels that Python does not resolve to supported codecs.

use astral_mail_headers::{DecodeError, Message};

#[test]
fn rejects_unknown_and_invalid_labels() {
    for label in [
        "utf.8",
        "utf..8",
        "utf.8.sig",
        "utf.16.le",
        "ascii.",
        "UTFK8",
        "UTFİ8",
        "utf\08",
    ] {
        let source = format!("X: =?{label}?Q?value?=");
        let message = Message::parse(source.as_bytes());
        assert_eq!(
            message.first("X").unwrap().decoded_value(),
            Err(DecodeError::UnsupportedCharset(label.to_owned())),
            "{label:?}"
        );
    }
}
