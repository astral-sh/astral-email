//! Compatibility with Python’s byte header parser.

#[path = "support/conformance.rs"]
mod conformance;

#[test]
fn matches_python_compat32() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/python.json")).unwrap();
    assert_eq!(fixtures["schema_version"], 1);
    assert_eq!(fixtures["python_version"], "3.12.13");
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
    }
}
