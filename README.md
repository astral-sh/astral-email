# astral-mail-headers

[![Crates.io](https://img.shields.io/crates/v/astral-mail-headers.svg)](https://crates.io/crates/astral-mail-headers)

A high-performance email header parser designed for Python packaging.

> [!WARNING]
>
> This project is developed with AI assistance, including its code and documentation.

## Benchmarks

### Resolution metadata

Parse package metadata, extract owned fields used for dependency resolution, and
drop the output (`astral_mail_headers::Message`).

| Parser              | maturin 1.4.0 | tqdm 4.66.1 | Black 24.8.0 |
| ------------------- | ------------: | ----------: | -----------: |
| astral-mail-headers |          1.51 |        3.98 |         3.30 |
| mailparse           |          6.55 |       18.21 |        12.61 |

<sub>Times in microseconds (µs);
[lower is better](https://github.com/astral-sh/astral-mail-headers/blob/dd7752820f4f2fb1005fd5bdb3646da20c36fc33/benchmarks/readme/README.md).</sub>

### Publishing metadata

Parse package metadata, extract owned fields and the description used for
publishing, and drop the output (`astral_mail_headers::Message`).

| Parser              | maturin 1.4.0 | tqdm 4.66.1 | Black 24.8.0 |
| ------------------- | ------------: | ----------: | -----------: |
| astral-mail-headers |          3.00 |        9.99 |        11.38 |
| mailparse           |         13.43 |       41.80 |        29.80 |

<sub>Times in microseconds (µs);
[lower is better](https://github.com/astral-sh/astral-mail-headers/blob/dd7752820f4f2fb1005fd5bdb3646da20c36fc33/benchmarks/readme/README.md).</sub>

### Wheel metadata

Parse WHEEL headers, collect owned names and decoded values, and drop the output
(`astral_mail_headers::Message`).

| Parser              | maturin 1.4.0 | tqdm 4.66.1 |
| ------------------- | ------------: | ----------: |
| astral-mail-headers |          0.55 |        0.57 |
| mailparse           |          1.37 |        1.19 |

<sub>Times in microseconds (µs);
[lower is better](https://github.com/astral-sh/astral-mail-headers/blob/dd7752820f4f2fb1005fd5bdb3646da20c36fc33/benchmarks/readme/README.md).</sub>

## Example usage

Use `Message` to parse `METADATA`, `PKG-INFO`, and `WHEEL` headers. Headers borrow
the input and preserve duplicates, spelling, folding, and encoded words. Decode
values explicitly when extracting metadata:

```rust
use astral_mail_headers::Message;

let source = b"Name: example\nRequires-Dist: requests>=2\n\nDescription\n";
let message = Message::parse(source);

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
and [uv integration](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/uv.md)
for details.

## Development

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

See the [fuzzing guide](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/fuzzing.md)
for additional validation.

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
