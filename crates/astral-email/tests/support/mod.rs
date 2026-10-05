use std::path::Path;

use astral_email::Message;
use mailparse::MailHeaderMap;
use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Kind {
    Resolution,
    Publish,
    Wheel,
}

pub(crate) struct Case {
    pub(crate) name: String,
    pub(crate) input: Vec<u8>,
    pub(crate) kind: Kind,
}

#[derive(Deserialize)]
pub(crate) struct Fixture {
    pub(crate) name: String,
    pub(crate) file: String,
    pub(crate) kind: Kind,
    pub(crate) headers: Vec<ExpectedHeader>,
    pub(crate) body_start: usize,
}

#[derive(Deserialize)]
pub(crate) struct ExpectedHeader {
    pub(crate) name: String,
    pub(crate) raw_value_hex: String,
}

pub(crate) fn fixtures() -> Vec<(Fixture, Vec<u8>)> {
    #[derive(Deserialize)]
    struct Manifest {
        cases: Vec<Fixture>,
    }
    let manifest: Manifest =
        serde_json::from_str(include_str!("../fixtures/uv/manifest.json")).unwrap();
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/uv");
    manifest
        .cases
        .into_iter()
        .map(|fixture| {
            let input = std::fs::read(directory.join(&fixture.file)).unwrap();
            (fixture, input)
        })
        .collect()
}

pub(crate) fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (fixture, input) in fixtures() {
        let kinds: &[Kind] = match fixture.kind {
            Kind::Wheel => &[Kind::Wheel],
            Kind::Resolution | Kind::Publish => &[Kind::Resolution, Kind::Publish],
        };
        for &kind in kinds {
            let suffix = match kind {
                Kind::Resolution => "resolution",
                Kind::Publish => "publish",
                Kind::Wheel => "wheel",
            };
            cases.push(Case {
                name: format!("{}-{suffix}", fixture.name),
                input: input.clone(),
                kind,
            });
        }
    }
    for count in [10, 100, 1_000] {
        let mut input = "Metadata-Version: 2.4\nName: demo\nVersion: 1.0\n".to_owned();
        for index in 0..count {
            input.push_str(&format!(
                "Requires-Dist: dependency-{index}>=1.0; python_version >= '3.10'\n"
            ));
        }
        cases.push(Case {
            name: format!("dependencies-{count}"),
            input: input.into_bytes(),
            kind: Kind::Resolution,
        });
    }
    for (name, value) in [
        ("folds", "A package\n with folded\n\tdescription text"),
        ("encoded-words", "=?utf-8?b?5Lit5paH?= =?utf-8?q?_package?="),
    ] {
        let mut input = "Metadata-Version: 2.4\nName: demo\nVersion: 1.0\n".to_owned();
        for _ in 0..100 {
            input.push_str(&format!("Classifier: {value}\n"));
        }
        cases.push(Case {
            name: name.to_owned(),
            input: input.into_bytes(),
            kind: Kind::Publish,
        });
    }
    let mut synthetic = |name: &str, value: &str, kind: Kind| {
        let field = match kind {
            Kind::Resolution => "Requires-Dist",
            Kind::Publish => "Classifier",
            Kind::Wheel => unreachable!(),
        };
        let input = format!(
            "Metadata-Version: 2.4\nName: demo\nVersion: 1.0\n{}",
            format!("{field}: {value}\n").repeat(32)
        );
        cases.push(Case {
            name: format!("synthetic-{name}"),
            input: input.into_bytes(),
            kind,
        });
    };
    for (name, value) in [
        ("plain-ascii-long", "a".repeat(4_096)),
        ("plain-utf8-long", "café 中文 ".repeat(512)),
        (
            "encoded-literal-prefix",
            format!("{} =?utf-8?q?caf=C3=A9?= suffix", "prefix ".repeat(64)),
        ),
        (
            "encoded-adjacent-same-label",
            "=?utf-8?q?caf=C3=A9?= =?UTF-8?q?_package?=".to_owned(),
        ),
        (
            "encoded-adjacent-different-label",
            "=?utf-8?q?caf=C3=A9?= =?iso-8859-1?q?_package?=".to_owned(),
        ),
        (
            "oversized-q-literal-heavy",
            format!("=?utf-8?q?{}=5F?=", "a".repeat(4_096)),
        ),
        (
            "oversized-q-hex-heavy",
            format!("=?utf-8?q?{}?=", "=61".repeat(4_096)),
        ),
        (
            "oversized-q-mixed",
            format!("=?utf-8?q?{}?=", "alpha_beta=3D".repeat(512)),
        ),
    ] {
        synthetic(name, &value, Kind::Publish);
    }
    synthetic(
        "dependency-equals",
        "dependency>=1.0,!=2.0,<=3.0; python_version >= '3.10' and os_name == 'posix'",
        Kind::Resolution,
    );
    for length in [1, 16, 64, 256, 4_096] {
        use base64::Engine;

        let payload = base64::engine::general_purpose::STANDARD.encode(vec![b'a'; length]);
        let name = if length >= 64 {
            format!("oversized-b-{length}")
        } else {
            format!("b-{length}")
        };
        synthetic(&name, &format!("=?utf-8?b?{payload}?="), Kind::Publish);
    }
    for (name, newline) in [("lf", "\n"), ("crlf", "\r\n")] {
        synthetic(
            &format!("fold-short-{name}"),
            &format!("short{newline} continuation"),
            Kind::Publish,
        );
        synthetic(
            &format!("fold-long-{name}"),
            &format!("{}end", format!("{}{newline} ", "a".repeat(128)).repeat(32)),
            Kind::Publish,
        );
    }
    cases
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Output {
    Metadata {
        fields: Vec<Vec<String>>,
        description: Option<String>,
    },
    Wheel(FxHashMap<String, Vec<String>>),
}

const RESOLUTION_FIELDS: &[(&str, bool)] = &[
    ("Name", false),
    ("Version", false),
    ("Requires-Dist", true),
    ("Requires-Python", false),
    ("Provides-Extra", true),
    ("Dynamic", true),
];

const PUBLISH_FIELDS: &[(&str, bool)] = &[
    ("Metadata-Version", false),
    ("Name", false),
    ("Version", false),
    ("Platform", true),
    ("Supported-Platform", true),
    ("Summary", false),
    ("Keywords", false),
    ("Home-Page", false),
    ("Download-URL", false),
    ("Author", false),
    ("Author-email", false),
    ("License", false),
    ("License-Expression", false),
    ("License-File", true),
    ("Classifier", true),
    ("Requires-Dist", true),
    ("Provides-Dist", true),
    ("Obsoletes-Dist", true),
    ("Maintainer", false),
    ("Maintainer-email", false),
    ("Requires-Python", false),
    ("Requires-External", true),
    ("Project-URL", true),
    ("Provides-Extra", true),
    ("Import-Name", true),
    ("Import-Namespace", true),
    ("Description-Content-Type", false),
    ("Dynamic", true),
];

fn metadata(kind: Kind, body: &[u8], values: impl Fn(&str, bool) -> Vec<String>) -> Output {
    let fields = match kind {
        Kind::Resolution => RESOLUTION_FIELDS,
        Kind::Publish => PUBLISH_FIELDS,
        Kind::Wheel => unreachable!(),
    };
    let description = if matches!(kind, Kind::Publish) {
        let body = str::from_utf8(body).unwrap();
        if body.trim().is_empty() {
            values("Description", false).into_iter().next()
        } else {
            Some(body.to_owned())
        }
    } else {
        None
    };
    Output::Metadata {
        fields: fields
            .iter()
            .map(|(name, all)| values(name, *all))
            .collect(),
        description,
    }
}

fn wheel(headers: impl Iterator<Item = (String, String)>) -> Output {
    let mut fields: FxHashMap<String, Vec<String>> = FxHashMap::default();
    for (name, mut value) in headers {
        let trimmed = value.trim();
        if trimmed.len() != value.len() {
            value = trimmed.to_owned();
        }
        fields.entry(name).or_default().push(value);
    }
    Output::Wheel(fields)
}

pub(crate) fn astral_email(input: &[u8], kind: Kind) -> Output {
    let message = Message::parse(input);
    if matches!(kind, Kind::Wheel) {
        return wheel(message.headers().iter().map(|header| {
            (
                header.name().to_owned(),
                header.decoded_value().unwrap().into_owned(),
            )
        }));
    }
    metadata(kind, message.body(), |name, all| {
        message
            .all(name)
            .take(if all { usize::MAX } else { 1 })
            .map(|header| header.decoded_value().unwrap().into_owned())
            .filter(|value| value != "UNKNOWN")
            .collect()
    })
}

pub(crate) fn mailparse(input: &[u8], kind: Kind) -> Output {
    let (headers, body_start) = mailparse::parse_headers(input).unwrap();
    if matches!(kind, Kind::Wheel) {
        return wheel(
            headers
                .iter()
                .map(|header| (header.get_key(), header.get_value())),
        );
    }
    metadata(kind, &input[body_start..], |name, all| {
        let values = if all {
            headers.get_all_values(name)
        } else {
            headers.get_first_value(name).into_iter().collect()
        };
        values
            .into_iter()
            .filter(|value| value != "UNKNOWN")
            .collect()
    })
}
