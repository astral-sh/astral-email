# astral-email

[![Crates.io](https://img.shields.io/crates/v/astral-email.svg)](https://crates.io/crates/astral-email)

A high-performance email header parser designed for Python packaging.

> [!WARNING]
>
> This project is developed with AI assistance, including its code and documentation.

## Example usage

Use `Message` to parse `METADATA`, `PKG-INFO`, and `WHEEL` headers. Headers borrow
the input and preserve duplicates, spelling, folding, and encoded words. Decode
values explicitly when extracting metadata:

```rust
use astral_email::Message;

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

See [conformance](https://github.com/astral-sh/astral-email/blob/main/docs/conformance.md),
[value decoding](https://github.com/astral-sh/astral-email/blob/main/docs/decoding.md),
and [uv integration](https://github.com/astral-sh/astral-email/blob/main/docs/uv.md)
for details.

## Development

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

See the [fuzzing guide](https://github.com/astral-sh/astral-email/blob/main/docs/fuzzing.md)
for additional validation.

## License

astral-email is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in astral-email by you, as defined in the Apache-2.0 license, shall
be dually licensed as above, without any additional terms or conditions.
