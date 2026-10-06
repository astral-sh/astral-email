#!/usr/bin/env python3
"""Compare exhaustive codec units and bounded headers with decode_inspect."""

import argparse
import base64
import itertools
import json
from pathlib import Path
import platform
import random
import subprocess
import sys


def word(charset: str, data: bytes, encoding: str) -> str:
    if encoding == "B":
        payload = base64.b64encode(data).decode("ascii")
    else:
        payload = "".join("_" if byte == 32 else f"={byte:02X}" for byte in data)
    return f"=?{charset}?{encoding}?{payload}?="


def codec_values():
    for charset in ["iso-8859-1", "windows-1252"]:
        for encoding in ["Q", "B"]:
            for byte in range(256):
                yield word(charset, bytes([byte]), encoding)
            yield word(charset, bytes(range(256)), encoding)

    boundaries = [0, 0x61, 0xD7FF, 0xD800, 0xDBFF, 0xDC00, 0xDFFF, 0xE000, 0xFEFF, 0xFFFE, 0xFFFF]
    for charset, byteorder in [("utf-16-le", "little"), ("utf-16-be", "big"), ("utf-16", sys.byteorder)]:
        # Every code unit in isolation, balancing Q and B transfer encodings.
        for unit in range(0x10000):
            yield word(charset, unit.to_bytes(2, byteorder), "Q" if unit % 2 else "B")

        # Exercise every high and low surrogate in a valid pair, including the
        # lowest and highest supplementary characters.
        for high in range(0xD800, 0xDC00):
            for low in [0xDC00, 0xDFFF]:
                data = high.to_bytes(2, byteorder) + low.to_bytes(2, byteorder)
                yield word(charset, data, "B")
        for low in range(0xDC00, 0xE000):
            for high in [0xD800, 0xDBFF]:
                data = high.to_bytes(2, byteorder) + low.to_bytes(2, byteorder)
                yield word(charset, data, "Q")

        # Adjacent boundary units test error consumption and BOM placement.
        for first, second in itertools.product(boundaries, repeat=2):
            data = first.to_bytes(2, byteorder) + second.to_bytes(2, byteorder)
            yield word(charset, data, "B")
        # A high surrogate followed by an incomplete unit is one Python error,
        # while a low surrogate followed by an incomplete unit is two errors.
        for unit, trailing in itertools.product(boundaries, range(256)):
            yield word(charset, unit.to_bytes(2, byteorder) + bytes([trailing]), "Q")

        # Explicit endianness preserves BOMs. Native UTF-16 consumes an initial
        # BOM and may switch byte order; a second BOM remains part of the text.
        for bom in [b"\xff\xfe", b"\xfe\xff"]:
            for data in [b"", b"a", b"a\x00", b"\x00a", b"\x00\xd8", b"\xd8\x00", b"\x00\xd8a", b"\xd8\x00a"]:
                for prefix in [bom, bom + bom]:
                    yield word(charset, prefix + data, "B")

        # Python joins adjacent encoded words before decoding the charset, even
        # when a BOM, code unit, or surrogate pair straddles the word boundary.
        for units in [(0x61,), (0xFEFF, 0x61), (0xFFFE, 0x61), (0xD800, 0xDC00), (0xD800, 0x61), (0xDC00,)]:
            data = b"".join(unit.to_bytes(2, byteorder) for unit in units)
            for suffix in [b"", b"a"]:
                raw = data + suffix
                for split in range(len(raw) + 1):
                    for separator in ["", " ", "\r\n\t"]:
                        yield word(charset, raw[:split], "Q") + separator + word(charset.upper(), raw[split:], "B")


def values():
    yield from codec_values()
    for separator in ["", " ", " text ", "\v"]:
        unknown = "=?unknown?q?a?="
        valid = "=?utf-8?q?b?="
        invalid = "=?utf-8?b?Y?="
        yield separator.join([unknown, invalid])
        yield separator.join([unknown, valid, invalid])
        yield separator.join([invalid, unknown])
        yield separator.join([unknown, valid])
    for separator in ["\v", "\f", "\x1c", "\x1d", "\x1e"]:
        yield f"before{separator} after"
        yield f"=?utf-8?q?hello?={separator}"
        yield f"=?utf-8?q?hello?={separator}  world"
        yield f"prefix{separator}=?utf-8?q?hello?="
        yield f"before{separator} after =?utf-8?q?end?="
        yield f"=?utf-8?q?=C3?={separator}=?utf-8?q?=A9?="
        yield f"=?utf-8?q?hello{separator}world?="
        yield f"=?utf{separator}-8?q?hello?="
    for charset in ["utf-8", "utf-8-sig"]:
        for encoding in ["Q", "B"]:
            for data in [b"\xef\xbb\xbftext", b"\xef\xbb\xbf\xef\xbb\xbftext", b"\xef\xbb"]:
                yield word(charset, data, encoding)
            for offset in [15, 16, 31, 32, 63, 64, 127, 128]:
                for invalid in [b"\xed\xa0\x80", b"\xf0\x80\x80A", b"\xf4\x90\x80\x80", b"\xf0\x9f\x92"]:
                    data = b"a" * offset + invalid
                    yield word(charset, data, encoding)
                    yield word(charset, data + b"z" * 128, encoding)
    randomizer = random.Random(0)
    charsets = [
        "utf-8", "utf-8-sig", "ascii", "iso-8859-1", "windows-1252",
        "utf-16", "utf-16-le", "utf-16-be",
    ]
    for index in range(5_000):
        charset = charsets[index % len(charsets)]
        data = randomizer.randbytes(randomizer.randrange(33))
        match index % 5:
            case 0:
                yield word(charset, data, "B")
            case 1:
                payload = "".join(randomizer.choices("AYZ019+/=! \t", k=randomizer.randrange(49)))
                yield f"=?{charset}?B?{payload}?="
            case 2:
                payload = "".join(randomizer.choices("abcDEF019_=G-! ", k=randomizer.randrange(49)))
                yield f"=?{charset}?Q?{payload}?="
            case 3:
                split = randomizer.randrange(len(data) + 1)
                separator = randomizer.choice(["", " ", "\t", "\r\n ", "\n\t"])
                yield word(charset, data[:split], "Q") + separator + word(charset.upper(), data[split:], "B")
            case 4:
                yield "  prefix " + word(charset, data, "Q") + " suffix  "


def main() -> None:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from generate_decode_fixtures import PYTHON_VERSION, inspect_value

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", help="path to the decode_inspect executable")
    args = parser.parse_args()
    if platform.python_implementation() != "CPython" or platform.python_version() != PYTHON_VERSION:
        parser.error(f"use CPython {PYTHON_VERSION}, found {platform.python_version()}")
    cases = list(values())
    result = subprocess.run(
        [args.binary],
        input="".join(json.dumps({"value": value}) + "\n" for value in cases),
        capture_output=True,
        text=True,
        check=True,
        timeout=30,
    )
    # JSON strings may contain Unicode line separators; JSONL uses literal LF.
    outputs = result.stdout.removesuffix("\n").split("\n")
    if len(outputs) != len(cases):
        parser.exit(1, f"Expected {len(cases)} results, received {len(outputs)}\n")
    errors = 0
    for index, (value, output) in enumerate(zip(cases, outputs, strict=True)):
        expected = inspect_value(value)
        actual = json.loads(output)
        if actual != expected:
            parser.exit(1, f"Case {index}: {value!r}\nPython: {expected!r}\nRust: {actual!r}\n")
        errors += "error" in expected
    print(f"Matched {len(cases)} cases ({errors} decoding errors), CPython {PYTHON_VERSION}, seed 0")


if __name__ == "__main__":
    main()
