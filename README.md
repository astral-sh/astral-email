# astral-email

A read-only parser for Python packaging headers, written in Rust.

The parser targets the header and body-boundary behavior of Python's `email.parser.BytesHeaderParser` with the `compat32` policy. It is intended for uv's `METADATA`, `PKG-INFO`, and `WHEEL` readers.

The application chooses the allocator. MIME body parsing, writing, and mutation are outside the library's scope.

## Development

```console
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

CI tests Linux AMD64 and ARM64 on Namespace runners.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
