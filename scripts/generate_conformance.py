#!/usr/bin/env python3
"""Generate byte-preserving fixtures from CPython's compat32 header parser."""

import argparse
from email.parser import BytesHeaderParser
from email.policy import compat32
import json
from pathlib import Path
import platform
import sys


PYTHON_VERSION = "3.12.13"
FIXTURES = (
    Path(__file__).resolve().parents[1]
    / "crates/astral-email/tests/fixtures/python.json"
)


def as_bytes(value: str) -> bytes:
    return value.encode("ascii", "surrogateescape")


def inspect_message(source: bytes) -> dict:
    message = BytesHeaderParser(policy=compat32).parsebytes(source)
    # get_payload() can replace non-ASCII bytes, even with decode=False.
    body = as_bytes(message._payload)
    unix_from = message.get_unixfrom()
    return {
        "headers": [
            {"name_hex": as_bytes(name).hex(), "value_hex": as_bytes(value).hex()}
            for name, value in message.raw_items()
        ],
        "body_hex": body.hex(),
        "body_offset": len(source) - len(body) if source.endswith(body) else None,
        "unix_from_hex": None if unix_from is None else as_bytes(unix_from).hex(),
        "defects": [type(defect).__name__ for defect in message.defects],
    }


def inputs():
    yield "empty", b""
    yield "body-only", b"A package description."
    yield "utf8-body", "\nA package with café and 中文.\n".encode()
    yield "binary-body", b"\n\x00\xff\x80\r\n"
    yield "opaque-multipart", (
        b"Content-Type: multipart/mixed; boundary=example\n\n"
        b"--example\nX: part\n\ncontents\n--example--\n"
    )
    yield "opaque-transfer-encoding", b"Content-Transfer-Encoding: base64\n\nZm9v\n"
    yield "duplicate-order", (
        b"Tag: first\ntAG: second\nOther: between\nTAG: third\n\nbody"
    )
    yield "encoded-words-retained", b"Summary: =?utf-8?q?caf=C3=A9?=\n\n"
    yield "utf8-values-retained", "Author: 中文\nSummary: café\n\n".encode()
    yield "binary-values-retained", b"Summary: \x00\x80\xff\n\n"

    # Exhaust the byte classes used to distinguish field names from body lines.
    for byte in range(256):
        yield f"field-name-byte-{byte:02x}", (
            b"X" + bytes([byte]) + b"Y: value\nNext: retained\n\nbody"
        )

    endings = [("lf", b"\n"), ("crlf", b"\r\n"), ("cr", b"\r")]
    for ending_name, eol in endings:
        for name, source in [
            ("blank", eol),
            ("blank-body", eol + b"body"),
            ("extra-blank", eol * 2 + b"body"),
            ("header-eof", b"Name: example"),
            ("header-terminated", b"Name: example" + eol),
            ("header-body", b"Name: example" + eol * 2 + b"body"),
            ("missing-separator", b"Name: example" + eol + b"description"),
            ("body-colon", b"Name: example" + eol * 2 + b"Another: body"),
            ("body-continuation", b"Name: example" + eol * 2 + b" body"),
            ("body-line-endings", b"Name: example" + eol * 2 + b"body\r\nnext\rlast\n"),
        ]:
            yield f"{ending_name}/{name}", source

        for name, first_line in [
            ("colonless", b"Description"),
            ("name-space", b"Bad Name: value"),
            ("name-tab", b"Bad\tName: value"),
            ("empty-name", b": value"),
            ("empty-field", b":"),
            ("space-continuation", b" orphan"),
            ("tab-continuation", b"\torphan"),
            ("whitespace-only", b" \t"),
        ]:
            for prefix_name, prefix in [("initial", b""), ("after-header", b"Name: example" + eol)]:
                yield f"{ending_name}/{prefix_name}/{name}", (
                    prefix + first_line + eol + b"Next: value" + eol * 2 + b"body"
                )

        for index, value in enumerate([
            b"", b" ", b"\t", b" \tvalue\t ", b"\vvalue\f", b"\x00value",
            b"value ", b"value\t", b"value: another", b"\xa0value", "é".encode(),
        ]):
            yield f"{ending_name}/value-{index}", b"X:" + value + eol * 2 + b"body"

        for name, lines in [
            ("fold-space", [b"X: first", b" second"]),
            ("fold-tab", [b"X: first", b"\tsecond"]),
            ("fold-mixed", [b"X: first ", b" \tsecond\t", b"  third"]),
            ("fold-leading", [b"X: \t", b" \t", b"  value"]),
            ("fold-empty", [b"X:", b" \t", b" "]),
            ("fold-encoded", [b"X: =?utf-8?Q?one?=", b" =?utf-8?Q?two?="]),
            ("orphan-then-fold", [b" orphan", b"X: first", b" second"]),
            ("empty-name-then-fold", [b": invalid", b" orphan", b"X: value"]),
            ("fold-then-missing-separator", [b"X: first", b" second", b"description"]),
        ]:
            yield f"{ending_name}/{name}", eol.join(lines) + eol * 2 + b"body"
            yield f"{ending_name}/{name}-eof", eol.join(lines)

        for name, lines in [
            ("envelope-only", [b"From sender"]),
            ("envelope-first", [b"From sender", b"X: value"]),
            ("envelope-middle", [b"X: first", b"From sender", b"Y: second"]),
            ("envelope-last", [b"X: first", b"From sender"]),
            ("envelope-repeated", [b"From first", b"From second"]),
            ("envelope-after-orphan", [b" orphan", b"From sender"]),
            ("envelope-before-fold", [b"X: first", b"From sender", b" orphan"]),
            ("envelope-before-empty-name", [b"X: first", b"From sender", b": invalid"]),
            ("envelope-missing-separator", [b"X: first", b"From sender", b"description"]),
        ]:
            yield f"{ending_name}/{name}", eol.join(lines) + eol * 2 + b"body"
            yield f"{ending_name}/{name}-eof", eol.join(lines)

    for first_name, first in endings:
        for second_name, second in endings:
            yield f"mixed/{first_name}-{second_name}", (
                b"X: first" + first + b" continuation" + second
                + b"Y: second" + first + second + b"body"
            )


def generate() -> dict:
    cases = []
    seen = set()
    for name, source in inputs():
        if source in seen:
            continue
        seen.add(source)
        cases.append({"name": name, "input_hex": source.hex(), **inspect_message(source)})
    return {
        "schema_version": 1,
        "python_version": PYTHON_VERSION,
        "parser": "BytesHeaderParser.parsebytes",
        "policy": "compat32",
        "cases": cases,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="check the committed fixtures")
    mode.add_argument("--stdin", action="store_true", help="read input_hex JSON lines and emit parsed JSON")
    args = parser.parse_args()
    if platform.python_implementation() != "CPython" or platform.python_version() != PYTHON_VERSION:
        parser.error(f"use CPython {PYTHON_VERSION}, found {platform.python_version()}")
    if args.stdin:
        for line in sys.stdin:
            request = json.loads(line)
            print(json.dumps(inspect_message(bytes.fromhex(request["input_hex"]))), flush=True)
        return
    corpus = generate()
    output = json.dumps(corpus, indent=2) + "\n"
    if args.check:
        if not FIXTURES.exists() or FIXTURES.read_text() != output:
            parser.exit(1, "Fixtures differ; run scripts/generate_conformance.py to regenerate.\n")
        print(f"Checked {len(corpus['cases'])} CPython {PYTHON_VERSION} cases")
    else:
        FIXTURES.parent.mkdir(parents=True, exist_ok=True)
        FIXTURES.write_text(output)
        print(f"Wrote {len(corpus['cases'])} cases to {FIXTURES}")


if __name__ == "__main__":
    main()
