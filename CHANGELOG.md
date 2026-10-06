# Changelog

## 0.0.1

Released on 2026-10-06.

### Enhancements

- Parse borrowed email headers while preserving order, duplicates, raw values, and the opaque body, with Python `compat32`-compatible recovery ([#2](https://github.com/astral-sh/astral-mail-headers/pull/2))
- Unfold header values and decode RFC 2047 Q and Base64 encoded words across supported character encodings ([#5](https://github.com/astral-sh/astral-mail-headers/pull/5))

### Bug fixes

- Match Python's line boundaries and whitespace handling when decoding encoded words ([#15](https://github.com/astral-sh/astral-mail-headers/pull/15))
- Recognize Python-normalized names and aliases for supported character encodings ([#16](https://github.com/astral-sh/astral-mail-headers/pull/16))
- Report invalid Base64 before unsupported character encodings when both errors occur, matching Python ([#20](https://github.com/astral-sh/astral-mail-headers/pull/20))

### Performance

- Reduce parsing and decoding overhead by delaying header-name conversion, consolidating value scans, and avoiding final line-ending rescans ([#48](https://github.com/astral-sh/astral-mail-headers/pull/48), [#49](https://github.com/astral-sh/astral-mail-headers/pull/49))
- Decode ordinary values faster with a single scan, SIMD UTF-8 validation, and fewer encoded-word candidate checks ([#24](https://github.com/astral-sh/astral-mail-headers/pull/24), [#23](https://github.com/astral-sh/astral-mail-headers/pull/23), [#58](https://github.com/astral-sh/astral-mail-headers/pull/58))
- Reduce encoded-word decoding allocations by borrowing payloads, reusing Base64 filtering storage, and writing Unicode escapes directly ([#25](https://github.com/astral-sh/astral-mail-headers/pull/25), [#45](https://github.com/astral-sh/astral-mail-headers/pull/45))
- Process heavily folded metadata faster by reserving unfolded values, skipping indentation in chunks, and handling continuation lines early ([#22](https://github.com/astral-sh/astral-mail-headers/pull/22), [#51](https://github.com/astral-sh/astral-mail-headers/pull/51), [#52](https://github.com/astral-sh/astral-mail-headers/pull/52))
- Preallocate eight header slots to avoid reallocation for typical package metadata, increasing initial allocation for smaller inputs ([#50](https://github.com/astral-sh/astral-mail-headers/pull/50))

### Other changes

- Publish the crate as `astral-mail-headers`, imported as `astral_mail_headers` ([#40](https://github.com/astral-sh/astral-mail-headers/pull/40))
