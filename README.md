# astral-mail-headers

[![Crates.io](https://img.shields.io/crates/v/astral-mail-headers.svg)](https://crates.io/crates/astral-mail-headers)

A high-performance email header parser designed for Python packaging.

> [!WARNING]
>
> This README was written by a human, but all code changes, PR summaries, and
> additional documentation were authored entirely by GPT-6 Astra in Codex.

## Benchmarks

### Dependency metadata extraction

Parse package metadata, extract owned fields used for dependency resolution, and
drop the output (`astral_mail_headers::Message`).

| Parser              | backcall | requests | Apache Airflow |  NumPy | pandas |
| ------------------- | -------: | -------: | -------------: | -----: | -----: |
| astral-mail-headers |     0.53 |     2.14 |          78.19 |   9.67 |  25.07 |
| mailparse           |     2.37 |     9.03 |         260.62 | 195.41 | 288.96 |

<sub>Times in microseconds (µs);
[lower is better](docs/performance.md#readme-results).</sub>

### Publication metadata extraction

Parse package metadata, extract owned fields and the description used for
publishing, and drop the output (`astral_mail_headers::Message`).

| Parser              | backcall | requests | Apache Airflow |  NumPy | pandas |
| ------------------- | -------: | -------: | -------------: | -----: | -----: |
| astral-mail-headers |     1.62 |     5.10 |          91.48 |  30.28 |  52.62 |
| mailparse           |     6.20 |    19.91 |         367.20 | 373.59 | 533.58 |

<sub>Times in microseconds (µs);
[lower is better](docs/performance.md#readme-results).</sub>

### WHEEL metadata extraction

Parse WHEEL headers, collect owned names and decoded values, and drop the output
(`astral_mail_headers::Message`).

| Parser              | backcall | requests | Apache Airflow | NumPy | pandas |
| ------------------- | -------: | -------: | -------------: | ----: | -----: |
| astral-mail-headers |     0.64 |     0.56 |           0.55 |  0.53 |   0.53 |
| mailparse           |     1.42 |     1.20 |           1.19 |  1.15 |   1.16 |

<sub>Times in microseconds (µs);
[lower is better](docs/performance.md#readme-results).</sub>

## Example usage

Use `Message` to parse `METADATA`, `PKG-INFO`, and `WHEEL` headers. Headers borrow
the input and preserve duplicates, spelling, folding, and encoded words. Decode
values when extracting metadata:

```rust
use astral_mail_headers::Message;

let message = Message::parse(b"Name: example\nRequires-Dist: requests>=2\n\nDescription\n");

assert_eq!(message.first("name").unwrap().raw_value(), b"example");

for header in message.all("Requires-Dist") {
    println!("{}", header.decoded_value()?);
}
```

Header parsing and malformed-input recovery follow Python's
`email.parser.BytesHeaderParser` with the `compat32` policy. The body remains
opaque; MIME parsing, writing, and mutation are outside the library's scope.

See [conformance](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/conformance.md),
[value decoding](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/decoding.md),
and [uv integration](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/uv.md).

## Development

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

See the [fuzzing guide](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/fuzzing.md).

## License

astral-mail-headers is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in astral-mail-headers by you, as defined in the Apache-2.0 license, shall
be dually licensed as above, without any additional terms or conditions.
