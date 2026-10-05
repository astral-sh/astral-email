//! Packaging fixtures and field extraction used by uv.

mod support;

use astral_mail_headers::Message;

#[test]
fn packaging_headers_match_python() {
    for (fixture, input) in support::fixtures() {
        let message = Message::parse(&input);
        assert_eq!(
            message.headers().len(),
            fixture.headers.len(),
            "{}",
            fixture.name
        );
        for (actual, expected) in message.headers().iter().zip(&fixture.headers) {
            let value: Vec<_> = expected
                .raw_value_hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|hex| u8::from_str_radix(str::from_utf8(hex).unwrap(), 16).unwrap())
                .collect();
            assert_eq!(actual.name(), expected.name, "{}", fixture.name);
            assert_eq!(
                actual.raw_value(),
                value,
                "{}: {}",
                fixture.name,
                expected.name
            );
        }
        assert_eq!(
            message.body(),
            &input[fixture.body_start..],
            "{}",
            fixture.name
        );
    }
}

#[test]
fn packaging_field_extraction_matches_mailparse() {
    for case in support::cases() {
        assert_eq!(
            support::astral_mail_headers(&case.input, case.kind),
            support::mailparse(&case.input, case.kind),
            "{}",
            case.name,
        );
    }
}
