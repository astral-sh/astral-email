#![no_main]

use astral_mail_headers::Message;
use libfuzzer_sys::fuzz_target;

fn offset(source: &[u8], field: &[u8]) -> usize {
    let start = (field.as_ptr() as usize)
        .checked_sub(source.as_ptr() as usize)
        .expect("field borrows the source");
    assert!(start <= source.len());
    assert!(field.len() <= source.len() - start);
    assert_eq!(field, &source[start..start + field.len()]);
    start
}

fuzz_target!(|source: &[u8]| {
    if source.len() > 16_384 {
        return;
    }
    let message = Message::parse(source);
    let mut previous_end = 0;
    for header in message.headers() {
        let name = header.name().as_bytes();
        assert!(!name.is_empty());
        assert!(
            name.iter()
                .all(|byte| (b'!'..=b'~').contains(byte) && *byte != b':')
        );
        let name_start = offset(source, name);
        let value_start = offset(source, header.raw_value());
        assert!(previous_end <= name_start);
        assert_eq!(source[name_start + name.len()], b':');
        assert!(name_start + name.len() < value_start);
        previous_end = value_start + header.raw_value().len();
    }
    if let Some(envelope) = message.unix_from() {
        assert_eq!(offset(source, envelope), 0);
        assert!(envelope.starts_with(b"From "));
    }
    assert!(message.body().len() <= source.len());
    let repeated = Message::parse(source);
    assert_eq!(message.headers(), repeated.headers());
    assert_eq!(message.body(), repeated.body());
    assert_eq!(message.unix_from(), repeated.unix_from());
    assert_eq!(message.defects(), repeated.defects());
});
