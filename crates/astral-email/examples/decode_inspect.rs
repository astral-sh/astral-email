//! JSONL adapter for bounded comparisons with the Python decoder oracle.

use std::io::{self, BufRead, Write};

use astral_email::{DecodeError, Message};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line?)?;
        let value = request["value"].as_str().ok_or("missing value string")?;
        if !value.is_ascii() {
            return Err("the Python decoder comparison requires ASCII input".into());
        }
        let source = format!("X:{value}");
        let message = Message::parse(source.as_bytes());
        let header = message.first("X").ok_or("missing wrapper header")?;
        let result = match header.decoded_value() {
            Ok(value) => json!({"value": value}),
            Err(DecodeError::InvalidBase64) => json!({"error": "invalid_base64"}),
            Err(DecodeError::UnsupportedCharset(_)) => {
                json!({"error": "unsupported_charset"})
            }
        };
        serde_json::to_writer(&mut output, &result)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
