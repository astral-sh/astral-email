#!/usr/bin/env python3
"""Apply the adapter to pinned uv and run upstream and adapter tests."""

import argparse
import hashlib
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
REVISION = "46b84fd0bfec23b72f29e8e2185ba68a65052f48"
TEST_MODULES = {
    "crates/uv-pypi-types/src/metadata/metadata_resolver.rs": "d588e0fd14ab84b4f588fb2d96eb2b41f76fb3f443b222fe09ff9f3b35e8b0b5",
    "crates/uv-pypi-types/src/metadata/metadata23.rs": "bc4b609350506075e273008e8236ed5fc42f57117bba8f4b39555d90f528f66f",
    "crates/uv-install-wheel/src/wheel.rs": "2ab2584a23790a010ae7362d37d42206ef620c2c143d4fdac3f3be4bdd45aaa8",
}


def check_test_modules(checkout):
    for relative, expected in TEST_MODULES.items():
        source = (checkout / relative).read_bytes()
        tests = source[source.index(b"#[cfg(test)]"):]
        if hashlib.sha256(tests).hexdigest() != expected:
            raise RuntimeError(f"upstream test module changed: {relative}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uv", type=Path, default=ROOT / "uv-source")
    parser.add_argument("--toolchain", help="Optional rustup toolchain for local validation")
    parser.add_argument("--apply-only", action="store_true")
    args = parser.parse_args()
    checkout = args.uv.resolve()
    if checkout.parent != ROOT:
        parser.error("the uv checkout must be a direct child of this repository")
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=checkout, text=True
    ).strip()
    if revision != REVISION:
        parser.error(f"expected uv {REVISION}, found {revision}")
    check_test_modules(checkout)

    patch = str(ROOT / "scripts/uv-tests/uv-integration.patch")
    applied = subprocess.run(
        ["git", "apply", "--reverse", "--check", patch],
        cwd=checkout,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0
    if not applied:
        subprocess.run(["git", "apply", "--check", patch], cwd=checkout, check=True)
        subprocess.run(["git", "apply", patch], cwd=checkout, check=True)
    check_test_modules(checkout)
    for crate in ("uv-pypi-types", "uv-install-wheel"):
        destination = checkout / "crates" / crate / "tests" / "astral_mail_headers.rs"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((ROOT / "scripts/uv-tests" / f"{crate}.rs").read_bytes())
    if args.apply_only:
        return

    cargo = ["cargo"]
    if args.toolchain:
        cargo.append(f"+{args.toolchain}")
    subprocess.run(
        cargo + [
            "test", "-p", "uv-pypi-types", "-p", "uv-install-wheel",
            "--lib", "--test", "astral_mail_headers",
        ],
        cwd=checkout,
        check=True,
    )


if __name__ == "__main__":
    main()
