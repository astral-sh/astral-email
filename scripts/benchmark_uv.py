#!/usr/bin/env python3
"""Build and run complete uv metadata parsing against the applied email adapter."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uv", type=Path, default=ROOT / "uv-source")
    parser.add_argument("--toolchain", help="Optional rustup toolchain for local builds")
    parser.add_argument("--allocator", choices=("system", "jemalloc"), default="system")
    parser.add_argument("--binary", type=Path, help="Copy the executable for interleaved baseline runs")
    parser.add_argument("--build-only", action="store_true")
    args, benchmark_args = parser.parse_known_args()
    checkout = args.uv.resolve()
    subprocess.run([
        "python3", str(ROOT / "scripts/test_uv.py"), "--uv", str(checkout), "--apply-only",
    ], check=True)
    crate = checkout / "crates/astral-email-uv-bench"
    (crate / "src").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / "scripts/uv-bench/Cargo.toml", crate / "Cargo.toml")
    shutil.copyfile(ROOT / "scripts/uv-bench/main.rs", crate / "src/main.rs")
    cargo = ["cargo"] + ([f"+{args.toolchain}"] if args.toolchain else [])
    command = cargo + ["build", "--release", "-p", "astral-email-uv-bench"]
    if args.allocator == "jemalloc":
        command += ["--features", "jemalloc"]
    subprocess.run(command, cwd=checkout, check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", checkout / "target"))
    binary = target.resolve() / "release/astral-email-uv-bench"
    if args.binary:
        shutil.copyfile(binary, args.binary)
        args.binary.chmod(0o755)
        binary = args.binary.resolve()
    if not args.build_only:
        if benchmark_args[:1] == ["--"]:
            benchmark_args = benchmark_args[1:]
        subprocess.run([
            str(binary), "--fixtures", str(ROOT / "crates/astral-email/tests/fixtures/uv"),
            *benchmark_args,
        ], check=True)


if __name__ == "__main__":
    main()
