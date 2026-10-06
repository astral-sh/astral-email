#!/usr/bin/env python3
"""Reproduce the frozen PyPI benchmark corpus, or validate its local bytes offline."""

import argparse
import email.parser
import email.policy
import hashlib
import json
from pathlib import Path
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / "crates/astral-mail-headers/tests/fixtures/packages"
MANIFEST = DESTINATION / "manifest.json"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def record(artifact, member, data):
    """Record exact archive bytes and Python's compatibility-mode header oracle."""
    name = member.rsplit("/", 1)[-1]
    package = f"{artifact['package']}-{artifact['version']}"
    is_metadata = name in ("METADATA", "PKG-INFO", "WHEEL")
    if is_metadata:
        filename = f"{package}.{name}"
    else:
        archive_kind = "wheel" if artifact["filename"].endswith(".whl") else "sdist"
        filename = f"licenses/{package}-{archive_kind}/{name}"
    result = {
        "file": filename,
        "package": artifact["package"],
        "version": artifact["version"],
        "source_url": artifact["url"],
        "source_sha256": artifact["sha256"],
        "member": member,
        "sha256": sha256(data),
        "total_bytes": len(data),
    }
    if is_metadata:
        message = email.parser.BytesHeaderParser(policy=email.policy.compat32).parsebytes(data)
        # The public payload accessor can replace non-ASCII bytes; retain the raw payload.
        body = message._payload.encode("ascii", "surrogateescape")
        body_start = len(data) - len(body)
        assert data[body_start:] == body
        headers = list(message.raw_items())
        result.update({
            "name": filename,
            "kind": "wheel" if name == "WHEEL" else "resolution",
            "headers": [
                {"name": key, "raw_value_hex": value.encode("ascii", "surrogateescape").hex()}
                for key, value in headers
            ],
            "body_start": body_start,
            "header_bytes": body_start,
            "header_count": len(headers),
            "requires_dist_count": sum(key.lower() == "requires-dist" for key, _ in headers),
        })
    return is_metadata, result


def archive_path(artifact, cache):
    path = cache / artifact["sha256"]
    if not path.exists():
        cache.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(artifact["url"], timeout=60) as response:
            data = response.read()
        if sha256(data) != artifact["sha256"]:
            raise ValueError(f"archive checksum mismatch: {artifact['filename']}")
        path.write_bytes(data)
    if sha256(path.read_bytes()) != artifact["sha256"]:
        raise ValueError(f"cached archive checksum mismatch: {path}")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate hashes, statistics, and oracle without network access")
    parser.add_argument("--cache", type=Path, default=Path.home() / ".cache/astral-mail-headers-packages")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text())
    cases, licenses = [], []
    existing = {
        (item["source_sha256"], item["member"]): item
        for item in manifest.get("cases", []) + manifest.get("licenses", [])
    }
    for artifact in manifest["artifacts"]:
        if args.check:
            archive = None
        else:
            path = archive_path(artifact, args.cache)
            archive = zipfile.ZipFile(path) if artifact["filename"].endswith(".whl") else tarfile.open(path)
        try:
            for member in artifact["members"]:
                if args.check:
                    expected = existing[(artifact["sha256"], member)]
                    data = (DESTINATION / expected["file"]).read_bytes()
                elif isinstance(archive, zipfile.ZipFile):
                    data = archive.read(member)
                else:
                    stream = archive.extractfile(member)
                    assert stream is not None
                    data = stream.read()
                is_metadata, item = record(artifact, member, data)
                (cases if is_metadata else licenses).append(item)
                if args.check:
                    if item != expected:
                        raise ValueError(f"fixture hash, statistics, or oracle mismatch: {item['file']}")
                else:
                    destination = DESTINATION / item["file"]
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(data)
        finally:
            if archive is not None:
                archive.close()
    if args.check:
        if cases != manifest["cases"] or licenses != manifest["licenses"]:
            raise ValueError("manifest records do not match the frozen archive members")
    else:
        manifest.update(cases=cases, licenses=licenses)
        MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"{'Checked' if args.check else 'Extracted'} {len(cases)} package fixtures and {len(licenses)} license/notice files")


if __name__ == "__main__":
    main()
