#!/usr/bin/env python3
"""Generate codec fixtures or inspect ASCII header values with CPython 3.12.13."""

import argparse
import base64
import codecs
from email.errors import HeaderParseError
from email.header import decode_header
from email.parser import BytesHeaderParser
from email.policy import compat32
import json
from pathlib import Path
import platform
import re
import sys


PYTHON_VERSION = "3.12.13"
FIXTURES = Path(__file__).resolve().parents[1] / "crates/astral-mail-headers/tests/fixtures/decode.json"
CODECS = {"ascii", "iso8859-1", "cp1252", "utf-8", "utf-8-sig", "utf-16", "utf-16-be", "utf-16-le"}


def decode_parts(parts: list) -> str:
    return "".join(
        part.decode(charset or "ascii", errors="replace") if isinstance(part, bytes) else part
        for part, charset in parts
    )


def generate() -> dict:
    inputs = [(charset, bytes(range(256))) for charset in ["ascii", "latin-1", "windows-1252"]]
    for charset in ["utf-8", "utf-16-le", "utf-16-be", "utf-16"]:
        for raw in [
            b"", b"\x00", b"\xff", b"\x00\xd8", b"\x00\xdc", b"\x00\xd8A\x00",
            b"\x00\xd8\x00\xdc", b"\xff\xfeA\x00", b"\xfe\xff\x00A",
            b"\xed\xa0\x80", b"\xf0\x80\x80A",
        ]:
            inputs.append((charset, raw))
    inputs.extend([
        (" UTF 8 ", b"\xc3\xa9"),
        ("---ASCII---", b"A\xff"),
        ("latin; 1", b"\xe9"),
        ("windows.1252", b"\x80"),
        ("ANSI.X3.4.1968", b"A\xff"),
        ("iso.8859.1", b"\xe9"),
        ("utf__8__sig", b"\xef\xbb\xbfA"),
        ("utf--16", b"\xff\xfeA\x00"),
        ("UTF 16 LE", b"A\x00"),
        ("utf.16be", b"\x00A"),
        ("utfÉ8", b"\xc3\xa9"),
        ("utf-☃-8", b"\xc3\xa9"),
        ("utf１６", b"\xc3\xa9"),
    ])
    cases = []
    for charset, raw in inputs:
        value = f"=?{charset}?B?{base64.b64encode(raw).decode('ascii')}?="
        cases.append({"value": value, "expected": decode_parts(decode_header(value))})
    return {
        "generator": f'CPython {PYTHON_VERSION}: email.header.decode_header; bytes.decode(charset, errors="replace")',
        "cases": cases,
    }


def inspect_value(value: str) -> dict:
    """Mirror the example's X-header wrapper, then apply the documented pipeline."""
    source = b"X:" + value.encode("ascii")
    message = BytesHeaderParser(policy=compat32).parsebytes(source)
    raw = next(value for name, value in message.raw_items() if name == "X")
    unfolded = re.sub(r"\r?\n[ \t]+", " ", raw)
    try:
        parts = decode_header(unfolded)
    except HeaderParseError:
        return {"error": "invalid_base64"}
    for _, charset in parts:
        if charset is not None:
            try:
                supported = codecs.lookup(charset).name in CODECS
            except (LookupError, ValueError):
                supported = False
            if not supported:
                return {"error": "unsupported_charset"}
    return {"value": decode_parts(parts)}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="check the committed fixtures")
    mode.add_argument("--stdin", action="store_true", help="read value JSON lines and emit decoded JSON")
    args = parser.parse_args()
    if platform.python_implementation() != "CPython" or platform.python_version() != PYTHON_VERSION:
        parser.error(f"use CPython {PYTHON_VERSION}, found {platform.python_version()}")
    if args.stdin:
        for line in sys.stdin:
            print(json.dumps(inspect_value(json.loads(line)["value"])), flush=True)
        return
    if sys.byteorder != "little":
        parser.error("fixture generation requires little-endian native UTF-16")
    corpus = generate()
    output = json.dumps(corpus, indent=2) + "\n"
    if args.check:
        if not FIXTURES.exists() or FIXTURES.read_text() != output:
            parser.exit(1, "Fixtures differ; run scripts/generate_decode_fixtures.py to regenerate.\n")
        print(f"Checked {len(corpus['cases'])} CPython {PYTHON_VERSION} decoder cases")
    else:
        FIXTURES.write_text(output)
        print(f"Wrote {len(corpus['cases'])} decoder cases to {FIXTURES}")


if __name__ == "__main__":
    main()
