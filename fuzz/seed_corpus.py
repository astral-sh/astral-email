#!/usr/bin/env python3
"""Seed fuzz targets from the Python, decoder and pinned uv fixtures."""

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGETS = ("reader", "decode", "lookup", "python", "python_decode")
MAX_BYTES = 16_384


def seed_inputs():
    inputs = {}
    report = {"fixture_sha256": {}, "excluded_over_size": 0}

    def read(path):
        data = path.read_bytes()
        report["fixture_sha256"][str(path.relative_to(ROOT))] = hashlib.sha256(data).hexdigest()
        return data

    def add(data):
        if len(data) > MAX_BYTES:
            report["excluded_over_size"] += 1
            return
        inputs[hashlib.sha256(data).hexdigest()] = data

    fixtures = ROOT / "crates/astral-email/tests/fixtures"
    cases = json.loads(read(fixtures / "python.json"))["cases"]
    report["python_inputs"] = len(cases)
    for case in cases:
        add(bytes.fromhex(case["input_hex"]))
        for header in case["headers"]:
            add(bytes.fromhex(header["value_hex"]))

    cases = json.loads(read(fixtures / "decode.json"))["cases"]
    report["decode_inputs"] = len(cases)
    for case in cases:
        value = case["value"].encode()
        add(value)
        add(b"X: " + value + b"\n\n")

    for value in [
        b"=?utf-8?q?hello?=\vworld",
        b"=?utf-8?q?hello\fworld?=",
        b"=?utf-8?q?=C3?=\x1c=?UTF-8?b?qQ==?=",
        b"=?utf-8?q?=C3?= =?latin-1?q?=A9?=",
        b"=?utf--8?q?text?=",
        b"=?windows.1252?q?=80?=",
        b"=?utf\x00-8?q?text?=",
        b"=?unknown?q?text?= ordinary =?utf-8?b?Y?=",
        b"prefix =?utf-8?x?literal?= =?utf-8?q?unfinished",
    ]:
        add(value)

    cases = json.loads(read(fixtures / "uv/manifest.json"))["cases"]
    report["uv_inputs"] = len(cases)
    for case in cases:
        data = read(fixtures / "uv" / case["file"])
        if hashlib.sha256(data).hexdigest() != case["sha256"]:
            raise ValueError(f"uv fixture hash changed: {case['file']}")
        add(data)
        add(data[:case["body_start"]])
        for header in case["headers"]:
            add(bytes.fromhex(header["raw_value_hex"]))

    report["unique_inputs"] = len(inputs)
    report["total_bytes"] = sum(map(len, inputs.values()))
    report["corpus_sha256"] = hashlib.sha256("\n".join(sorted(inputs)).encode()).hexdigest()
    return inputs, report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("targets", nargs="*", choices=TARGETS)
    parser.add_argument("--output", type=Path, default=ROOT / "fuzz/generated")
    args = parser.parse_args()
    inputs, report = seed_inputs()
    targets = args.targets or TARGETS
    for target in targets:
        directory = args.output / target
        directory.mkdir(parents=True, exist_ok=True)
        for digest, data in sorted(inputs.items()):
            (directory / digest).write_bytes(data)
    report["targets"] = list(targets)
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
