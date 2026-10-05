//! JSONL adapter for differential tests against Python.

#[path = "../tests/support/conformance.rs"]
mod conformance;

use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line?)?;
        let source = conformance::decode_hex(
            request["input_hex"]
                .as_str()
                .ok_or("missing input_hex string")?,
        )?;
        serde_json::to_writer(&mut output, &conformance::inspect(&source))?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
