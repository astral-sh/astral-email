#!/usr/bin/env python3
"""Compare bounded Q/B and separator cases with the compiled decode_inspect example."""

import argparse
import base64
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


def values():
    for separator in ["\v", "\f", "\x1c", "\x1d", "\x1e"]:
        yield f"before{separator} after"
        yield f"=?utf-8?q?hello?={separator}"
        yield f"=?utf-8?q?hello?={separator}  world"
        yield f"prefix{separator}=?utf-8?q?hello?="
        yield f"before{separator} after =?utf-8?q?end?="
        yield f"=?utf-8?q?=C3?={separator}=?utf-8?q?=A9?="
        yield f"=?utf-8?q?hello{separator}world?="
        yield f"=?utf{separator}-8?q?hello?="
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
