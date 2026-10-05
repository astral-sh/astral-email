//! Exercise the email adapter through uv's public WHEEL parser.

use uv_install_wheel::{Error, WheelFile};

#[test]
fn all_wheel_headers_are_decoded_with_error_context() {
    for field in ["Wheel-Version", "Tag", "X-Unused"] {
        for (value, detail) in [
            (
                "=?x-unknown?q?text?=",
                "unsupported header charset: x-unknown",
            ),
            ("=?utf-8?b?A?=", "invalid Base64 in encoded header word"),
        ] {
            let source = format!("Wheel-Version: 1.0\n{field}: {value}\n");
            let Error::InvalidWheel(message) = WheelFile::parse(&source).unwrap_err() else {
                panic!("expected an invalid wheel error");
            };
            assert_eq!(message, format!("Failed to decode `WHEEL` file: {detail}"));
        }
    }
}

#[test]
fn tags_keep_spelling_order_and_unknown_values() {
    let wheel = WheelFile::parse(
        "Wheel-Version: 1.0\n\
         Wheel-Version: 2.0\n\
         Tag: \t=?utf-8?q?py3-none-any?= \t\n\
         tag: ignored\n\
         Tag: UNKNOWN\n\
         Tag: cp312-none-any\n",
    )
    .unwrap();
    assert_eq!(
        wheel.tags().unwrap(),
        ["py3-none-any", "UNKNOWN", "cp312-none-any"]
    );
    assert!(matches!(
        WheelFile::parse("wheel-version: 1.0\n"),
        Err(Error::InvalidWheel(_))
    ));
}

#[test]
fn malformed_lines_stop_wheel_headers() {
    for line in ["colonless", "Bad Name: value"] {
        let source = format!("Wheel-Version: 1.0\nTag: py3-none-any\n{line}\nTag: =?utf-8?b?A?=\n");
        let wheel = WheelFile::parse(&source).unwrap();
        assert_eq!(wheel.tags().unwrap(), ["py3-none-any"]);
        assert!(matches!(
            WheelFile::parse(&format!("{line}\nWheel-Version: 1.0\n")),
            Err(Error::InvalidWheel(_))
        ));
    }
}
