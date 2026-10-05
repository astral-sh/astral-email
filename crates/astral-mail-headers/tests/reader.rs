//! Borrowing, lookup and recovery contracts of the header reader.

use astral_mail_headers::{Defect, Message};

#[test]
fn preserves_order_and_borrows_values() {
    let source = b"Tag: first\r\ntag: second\r\nOther: value\r\n\r\nbody";
    let message = Message::parse(source);
    let values: Vec<_> = message
        .all("TAG")
        .map(|header| header.raw_value())
        .collect();
    assert_eq!(values, [b"first".as_slice(), b"second"]);
    assert_eq!(message.first("tag").unwrap().name(), "Tag");
    assert_eq!(
        message.headers()[0].raw_value().as_ptr(),
        source[5..].as_ptr()
    );
    assert_eq!(message.body().as_ptr(), source[source.len() - 4..].as_ptr());
    assert!(message.defects().is_empty());
}

#[test]
fn preserves_folding_and_encoded_words() {
    let message = Message::parse(b"Name: \r\n\t =?utf-8?q?demo?=\r\n next \r\n\r\n");
    assert_eq!(
        message.first("name").unwrap().raw_value(),
        b"=?utf-8?q?demo?=\r\n next "
    );
}

#[test]
fn stops_at_invalid_names_and_records_python_defect_order() {
    let message =
        Message::parse(b" continuation\n: empty name\nName: x\nbad name: body\nVersion: 1");
    assert_eq!(message.headers().len(), 1);
    assert_eq!(message.body(), b"bad name: body\nVersion: 1");
    assert_eq!(
        message.defects(),
        [
            Defect::MissingHeaderBodySeparator,
            Defect::FirstHeaderLineIsContinuation,
            Defect::InvalidHeader
        ]
    );
}

#[test]
fn recovers_trailing_envelope_into_body() {
    let message = Message::parse(b"From sender\nName: x\nFrom tail\n\nbody");
    assert_eq!(message.unix_from(), Some(b"From sender".as_slice()));
    assert_eq!(message.body(), b"From tail\nbody");
    assert!(message.defects().is_empty());
}

#[test]
fn accepts_empty_input_and_eof_without_newline() {
    assert!(Message::parse(b"").headers().is_empty());
    let message = Message::parse(b"Name: x");
    assert_eq!(message.first("name").unwrap().raw_value(), b"x");
    assert!(message.body().is_empty());
}
