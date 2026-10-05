use astral_mail_headers::Message;
use serde_json::{Value, json};

pub(super) fn decode_hex(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !input.is_ascii() || !input.len().is_multiple_of(2) {
        return Err("expected an even number of hexadecimal digits".into());
    }
    (0..input.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&input[index..index + 2], 16).map_err(Into::into))
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|byte| [HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]])
        .map(char::from)
        .collect()
}

pub(super) fn inspect(source: &[u8]) -> Value {
    let message = Message::parse(source);
    let headers: Vec<_> = message
        .headers()
        .iter()
        .map(|header| {
            json!({
                "name_hex": encode_hex(header.name().as_bytes()),
                "value_hex": encode_hex(header.raw_value()),
            })
        })
        .collect();
    let body_offset = source
        .ends_with(message.body())
        .then(|| source.len() - message.body().len());
    let defects: Vec<_> = message
        .defects()
        .iter()
        .map(|defect| format!("{defect:?}Defect"))
        .collect();
    json!({
        "headers": headers,
        "body_hex": encode_hex(message.body()),
        "body_offset": body_offset,
        "unix_from_hex": message.unix_from().map(encode_hex),
        "defects": defects,
    })
}
