#!/usr/bin/env python3
"""Extract packaging metadata from a checkout of the pinned uv revision."""

import argparse
import email.parser
import email.policy
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import zipfile

REVISION = "46b84fd0bfec23b72f29e8e2185ba68a65052f48"
ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / "crates/astral-email/tests/fixtures/uv"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    args = parser.parse_args()
    DESTINATION.mkdir(parents=True, exist_ok=True)
    cases = []

    def add(source, member, content, kind, filename):
        message = email.parser.BytesHeaderParser(policy=email.policy.compat32).parsebytes(content)
        # get_payload() decodes unknown body bytes; retain the parser's raw payload.
        body = message._payload.encode("ascii", "surrogateescape")
        body_start = len(content) - len(body)
        assert content[body_start:] == body
        (DESTINATION / filename).write_bytes(content)
        cases.append({
            "name": filename,
            "file": filename,
            "kind": kind,
            "source": source.relative_to(args.checkout).as_posix(),
            "member": member,
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "sha256": hashlib.sha256(content).hexdigest(),
            "headers": [
                {"name": name, "raw_value_hex": value.encode("ascii", "surrogateescape").hex()}
                for name, value in message.raw_items()
            ],
            "body_start": body_start,
        })

    for source in sorted((args.checkout / "test/links").iterdir()):
        if source.suffix == ".whl":
            with zipfile.ZipFile(source) as archive:
                for member in sorted(archive.namelist()):
                    if member.endswith((".dist-info/METADATA", ".dist-info/WHEEL")):
                        kind = "wheel" if member.endswith("/WHEEL") else "resolution"
                        filename = source.name + "." + member.replace("/", "__")
                        add(source, member, archive.read(member), kind, filename)
        elif tarfile.is_tarfile(source):
            with tarfile.open(source) as archive:
                for member in sorted(archive.getmembers(), key=lambda member: member.name):
                    if member.isfile() and member.name.endswith("/PKG-INFO"):
                        stream = archive.extractfile(member)
                        assert stream is not None
                        filename = source.name + "." + member.name.replace("/", "__")
                        add(source, member.name, stream.read(), "publish", filename)

    source = args.checkout / "test/ecosystem/black/PKG-INFO"
    add(source, None, source.read_bytes(), "publish", "black-24.8.0.PKG-INFO")
    for filename in ("LICENSE-MIT", "LICENSE-APACHE"):
        shutil.copyfile(args.checkout / filename, DESTINATION / filename)
    shutil.copyfile(args.checkout / "test/ecosystem/black/LICENSE", DESTINATION / "LICENSE-black")
    manifest = {"repository": "https://github.com/astral-sh/uv", "revision": REVISION, "cases": cases}
    (DESTINATION / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Extracted {len(cases)} metadata files")


if __name__ == "__main__":
    main()
