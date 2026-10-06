# astral-mail-headers

[![Crates.io](https://img.shields.io/crates/v/astral-mail-headers.svg)](https://crates.io/crates/astral-mail-headers)

A high-performance email header parser designed for Python packaging.

> [!WARNING]
>
> This project is developed with AI assistance, including its code and documentation.

## Benchmarks

All performance benchmarks use unmodified `METADATA`, `PKG-INFO`, and `WHEEL`
files from published Python packages: backcall, six, packaging, requests, Black,
setuptools, NumPy, pandas, and Apache Airflow. The corpus covers small wheel
headers, ordinary package metadata, long folded licenses, large descriptions,
and hundreds of dependencies.

We measure parsing, owned field extraction for resolution and publishing, and
allocations. The wall-clock benchmark compares against mailparse with the system
allocator or jemalloc. See the [corpus and benchmark commands](docs/performance.md).

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
