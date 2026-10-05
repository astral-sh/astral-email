//! Exercise the email adapter through uv's public metadata parsers.

use std::fmt::Debug;

use astral_email::DecodeError;
use uv_pypi_types::{Metadata10, Metadata23, MetadataError, ResolutionMetadata};

const HEADERS: &str = "Metadata-Version: 2.3\nName: demo\nVersion: 1.0\n";

fn decode_error<T: Debug>(result: Result<T, MetadataError>) -> DecodeError {
    match result.unwrap_err() {
        MetadataError::Decode(error) => error,
        error => panic!("expected a decoding error, got {error:?}"),
    }
}

#[test]
fn required_values_propagate_decoding_errors() {
    for (value, expected) in [
        (
            "=?x-unknown?q?demo?=",
            DecodeError::UnsupportedCharset("x-unknown".to_owned()),
        ),
        ("=?utf-8?b?A?=", DecodeError::InvalidBase64),
    ] {
        let source = format!("Metadata-Version: 2.3\nName: {value}\nVersion: 1.0\n");
        assert_eq!(
            decode_error(Metadata10::parse_pkg_info(source.as_bytes())),
            expected
        );
        assert_eq!(
            decode_error(ResolutionMetadata::parse_metadata(source.as_bytes())),
            expected
        );
        assert_eq!(
            decode_error(ResolutionMetadata::parse_pkg_info(source.as_bytes())),
            expected
        );
        assert_eq!(decode_error(Metadata23::parse(source.as_bytes())), expected);
    }
}

#[test]
fn repeated_values_propagate_decoding_errors() {
    for field in ["Requires-Dist", "Provides-Extra", "Dynamic"] {
        let source = format!("{HEADERS}{field}: idna\n{field}: =?utf-8?b?A?=\n");
        assert_eq!(
            decode_error(ResolutionMetadata::parse_metadata(source.as_bytes())),
            DecodeError::InvalidBase64,
        );
        assert_eq!(
            decode_error(ResolutionMetadata::parse_pkg_info(source.as_bytes())),
            DecodeError::InvalidBase64,
        );
        assert_eq!(
            decode_error(Metadata23::parse(source.as_bytes())),
            DecodeError::InvalidBase64
        );
    }
}

#[test]
fn unconsumed_values_are_not_decoded() {
    let source = format!("{HEADERS}Summary: =?utf-8?b?A?=\nX-Unused: =?x-unknown?q?text?=\n");
    assert_eq!(
        Metadata10::parse_pkg_info(source.as_bytes())
            .unwrap()
            .name
            .as_str(),
        "demo"
    );
    assert_eq!(
        ResolutionMetadata::parse_metadata(source.as_bytes())
            .unwrap()
            .name
            .as_str(),
        "demo"
    );
    assert_eq!(
        ResolutionMetadata::parse_pkg_info(source.as_bytes())
            .unwrap()
            .name
            .as_str(),
        "demo"
    );
    assert_eq!(
        decode_error(Metadata23::parse(source.as_bytes())),
        DecodeError::InvalidBase64
    );
}

#[test]
fn first_values_and_unknown_filtering_keep_uv_policy() {
    let source = format!(
        "{HEADERS}\
         nAmE: =?utf-8?b?A?=\n\
         Version: 2.0\n\
         Requires-Python: UNKNOWN\n\
         Requires-Python: >=3.10\n\
         Summary: UNKNOWN\n\
         Summary: =?utf-8?b?A?=\n\
         Requires-Dist: UNKNOWN\n\
         Requires-Dist: idna\n\
         Requires-Dist: requests\n\
         Provides-Extra: UNKNOWN\n\
         Provides-Extra: test\n\
         Provides-Extra: test\n"
    );
    let identity = Metadata10::parse_pkg_info(source.as_bytes()).unwrap();
    assert_eq!(identity.name.as_str(), "demo");
    assert_eq!(identity.version, "1.0");
    for metadata in [
        ResolutionMetadata::parse_metadata(source.as_bytes()).unwrap(),
        ResolutionMetadata::parse_pkg_info(source.as_bytes()).unwrap(),
    ] {
        assert_eq!(metadata.name.as_str(), "demo");
        assert_eq!(metadata.version.to_string(), "1.0");
        assert!(metadata.requires_python.is_none());
        assert_eq!(
            metadata
                .requires_dist
                .iter()
                .map(|requirement| requirement.name.as_str())
                .collect::<Vec<_>>(),
            ["idna", "requests"]
        );
        assert_eq!(
            metadata
                .provides_extra
                .iter()
                .map(|extra| extra.as_str())
                .collect::<Vec<_>>(),
            ["test", "test"]
        );
    }
    let metadata = Metadata23::parse(source.as_bytes()).unwrap();
    assert_eq!(metadata.name, "demo");
    assert_eq!(metadata.version, "1.0");
    assert!(metadata.requires_python.is_none());
    assert!(metadata.summary.is_none());
    assert_eq!(metadata.requires_dist, ["idna", "requests"]);
    assert_eq!(metadata.provides_extra, ["test", "test"]);
}

#[test]
fn unknown_required_value_does_not_fall_back_to_a_duplicate() {
    let source = b"Metadata-Version: 2.3\nName: UNKNOWN\nName: demo\nVersion: 1.0\n";
    assert!(matches!(
        Metadata10::parse_pkg_info(source),
        Err(MetadataError::FieldNotFound("Name"))
    ));
    assert!(matches!(
        ResolutionMetadata::parse_metadata(source),
        Err(MetadataError::FieldNotFound("Name"))
    ));
    assert!(matches!(
        ResolutionMetadata::parse_pkg_info(source),
        Err(MetadataError::FieldNotFound("Name"))
    ));
    assert!(matches!(
        Metadata23::parse(source),
        Err(MetadataError::FieldNotFound("Name"))
    ));
}

#[test]
fn malformed_lines_end_headers_and_become_the_description() {
    for line in ["colonless", "Bad Name: value"] {
        let body = format!("{line}\nRequires-Dist: idna\n");
        let source = format!("{HEADERS}Description: fallback\n{body}");
        assert!(
            ResolutionMetadata::parse_metadata(source.as_bytes())
                .unwrap()
                .requires_dist
                .is_empty()
        );
        assert!(
            ResolutionMetadata::parse_pkg_info(source.as_bytes())
                .unwrap()
                .requires_dist
                .is_empty()
        );
        let metadata = Metadata23::parse(source.as_bytes()).unwrap();
        assert!(metadata.requires_dist.is_empty());
        assert_eq!(metadata.description.as_deref(), Some(body.as_str()));
    }
}

#[test]
fn leading_continuations_and_lone_cr_are_recovered() {
    let source = b" orphan\rMetadata-Version: 2.3\rName: demo\rVersion: 1.0\r";
    assert_eq!(
        Metadata10::parse_pkg_info(source).unwrap().name.as_str(),
        "demo"
    );
    assert_eq!(
        ResolutionMetadata::parse_metadata(source)
            .unwrap()
            .name
            .as_str(),
        "demo"
    );
    assert_eq!(
        ResolutionMetadata::parse_pkg_info(source)
            .unwrap()
            .name
            .as_str(),
        "demo"
    );
    assert_eq!(Metadata23::parse(source).unwrap().name, "demo");
}

#[test]
fn description_fallback_only_decodes_a_used_header() {
    let source = format!("{HEADERS}Description: =?utf-8?b?A?=\n\n body\r\n ");
    assert_eq!(
        Metadata23::parse(source.as_bytes())
            .unwrap()
            .description
            .as_deref(),
        Some(" body\r\n ")
    );

    let source = format!("{HEADERS}Description: fallback\r\n text\r\n\r\n \t\r\n");
    assert_eq!(
        Metadata23::parse(source.as_bytes())
            .unwrap()
            .description
            .as_deref(),
        Some("fallback text")
    );

    let source = format!("{HEADERS}Description: =?utf-8?b?A?=\n\n \t\n");
    assert_eq!(
        decode_error(Metadata23::parse(source.as_bytes())),
        DecodeError::InvalidBase64
    );
}

#[test]
fn only_publishing_validates_description_utf8() {
    let mut source = format!("{HEADERS}Description: fallback\n\n").into_bytes();
    source.push(0xff);
    assert!(Metadata10::parse_pkg_info(&source).is_ok());
    assert!(ResolutionMetadata::parse_metadata(&source).is_ok());
    assert!(ResolutionMetadata::parse_pkg_info(&source).is_ok());
    assert!(matches!(
        Metadata23::parse(&source),
        Err(MetadataError::DescriptionEncoding(_))
    ));
}

#[test]
fn publication_index_preserves_order_and_ignores_unknown_fields() {
    let source = format!(
        "{HEADERS}\
         cLaSsIfIeR: first\n\
         X-Unknown: =?utf-8?b?A?=\n\
         Requires-Dist: idna\n\
         CLASSIFIER: UNKNOWN\n\
         Requires-Dist: requests\n\
         Classifier: third\n"
    );
    let metadata = Metadata23::parse(source.as_bytes()).unwrap();
    assert_eq!(metadata.classifiers, ["first", "third"]);
    assert_eq!(metadata.requires_dist, ["idna", "requests"]);

    let source = format!("{HEADERS}Requires-Dist: =?utf-8?b?A?=\nSummary: =?x-unknown?q?text?=\n");
    assert_eq!(
        decode_error(Metadata23::parse(source.as_bytes())),
        DecodeError::UnsupportedCharset("x-unknown".to_owned()),
    );
}

#[test]
fn repeated_decoding_precedes_semantic_errors_and_short_circuiting() {
    for fields in [
        "Requires-Dist: @invalid\nRequires-Dist: =?utf-8?b?A?=\n",
        "Dynamic: Version\nDynamic: =?utf-8?b?A?=\n",
        "Dynamic: Requires-Python\nDynamic: =?utf-8?b?A?=\n",
    ] {
        let source = format!("{HEADERS}{fields}");
        assert_eq!(
            decode_error(ResolutionMetadata::parse_metadata(source.as_bytes())),
            DecodeError::InvalidBase64,
        );
        assert_eq!(
            decode_error(ResolutionMetadata::parse_pkg_info(source.as_bytes())),
            DecodeError::InvalidBase64,
        );
    }
}
