# astral-email

A read-only parser for Python packaging headers, written in Rust.

The parser targets the header and body-boundary behavior of Python's `email.parser.BytesHeaderParser` with the `compat32` policy. It is intended for uv's `METADATA`, `PKG-INFO`, and `WHEEL` readers.

The application chooses the allocator. MIME body parsing, writing, and mutation are outside the library's scope.

```rust
use astral_email::Message;

let message = Message::parse(b"Name: example\nRequires-Dist: requests>=2\n\nDescription\n");
assert_eq!(message.first("name").unwrap().raw_value(), b"example");

for header in message.all("Requires-Dist") {
    println!("{}", header.decoded_value().unwrap());
}
```

Headers borrow the input and preserve duplicates, spelling, folding, and encoded
words. Malformed headers recover as Python does, with defects available to the
caller. Text decoding is explicit and fallible; the supported codecs and
differences from Python's text conversion are documented separately.

- [Conformance](docs/conformance.md) and [value decoding](docs/decoding.md)
- [uv integration](docs/uv.md)
- [Benchmarks](docs/performance.md)
- [Fuzzing](docs/fuzzing.md)

## Development

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

CI tests Linux AMD64 and ARM64 on GitHub-hosted runners.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
