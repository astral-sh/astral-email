# astral-mail-headers

[![Crates.io](https://img.shields.io/crates/v/astral-mail-headers.svg)](https://crates.io/crates/astral-mail-headers)

A high-performance email header parser designed for Python packaging.

> [!WARNING]
>
> This project is developed with AI assistance, including its code and documentation.

## Benchmarks

Benchmarks use unmodified `METADATA`, `PKG-INFO`, and `WHEEL` files from published
Python packages. They measure raw header parsing and owned dependency, publication,
and wheel metadata extraction. See [Benchmarks](docs/performance.md) for the corpus,
allocator comparisons, and commands.

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

See [conformance](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/conformance.md)
and [value decoding](https://github.com/astral-sh/astral-mail-headers/blob/main/docs/decoding.md).

## Development

See [Contributing](https://github.com/astral-sh/astral-mail-headers/blob/main/CONTRIBUTING.md)
for tests, fuzzing, and releases.

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
