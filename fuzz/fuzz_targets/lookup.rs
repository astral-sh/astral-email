#![no_main]

use astral_email::Message;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|source: &[u8]| {
    if source.len() > 16_384 {
        return;
    }
    let message = Message::parse(source);
    let mut queries = vec!["Name".to_owned(), "Requires-Dist".to_owned(), String::new()];
    if let Some(first) = message.headers().first() {
        queries.push(first.name().to_ascii_lowercase());
    }
    if let Some(last) = message.headers().last() {
        queries.push(last.name().to_ascii_uppercase());
    }
    queries.push(String::from_utf8_lossy(source).into_owned());
    for query in queries {
        let expected: Vec<_> = message
            .headers()
            .iter()
            .filter(|header| header.name().eq_ignore_ascii_case(&query))
            .collect();
        assert_eq!(
            message.first(&query).map(std::ptr::from_ref),
            expected.first().map(|header| std::ptr::from_ref(*header))
        );
        let actual: Vec<_> = message.all(&query).collect();
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(std::ptr::eq(actual, expected));
        }
    }
});
