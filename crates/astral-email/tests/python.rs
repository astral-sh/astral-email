//! Compatibility with Python’s byte header parser.

#[path = "support/conformance.rs"]
mod conformance;

use astral_email::{Message, Parser};

#[test]
fn matches_python_compat32() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/python.json")).unwrap();
    assert_eq!(fixtures["schema_version"], 1);
    assert_eq!(fixtures["python_version"], "3.12.13");
    let mut parser = Parser::default();
    for case in fixtures["cases"].as_array().unwrap() {
        let source = conformance::decode_hex(case["input_hex"].as_str().unwrap()).unwrap();
        let mut expected = case.as_object().unwrap().clone();
        expected.remove("name");
        expected.remove("input_hex");
        assert_eq!(
            conformance::inspect(&source),
            serde_json::Value::Object(expected),
            "{}",
            case["name"]
        );
        let fresh = Message::parse(&source);
        let reused = parser.parse(&source);
        assert!(fresh.headers().iter().copied().eq(reused.headers()));
        assert_eq!(fresh.body(), reused.body());
        assert_eq!(fresh.unix_from(), reused.unix_from());
        assert_eq!(fresh.defects(), reused.defects());
        for header in fresh.headers() {
            let name = header.name().to_ascii_lowercase();
            assert_eq!(fresh.first(&name).copied(), reused.first(&name));
            assert!(fresh.all(&name).copied().eq(reused.all(&name)));
        }
        let empty = parser.parse(b"");
        assert_eq!(empty.headers().len(), 0);
        assert!(empty.body().is_empty());
        assert!(empty.unix_from().is_none());
        assert!(empty.defects().is_empty());
    }
}
