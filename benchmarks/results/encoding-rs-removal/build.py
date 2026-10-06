"""Build comparison executables: python3 build.py WORKTREE baseline|candidate."""

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SOURCE_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/astral-mail-headers/Cargo.toml",
    "crates/astral-mail-headers/src/decode.rs",
    "crates/astral-mail-headers/benches/metadata-extraction.rs",
]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    repo = Path(sys.argv[1]).resolve()
    label = sys.argv[2]
    assert label in ("baseline", "candidate")
    (ROOT / "bin").mkdir(exist_ok=True)
    env = os.environ.copy()
    env.update(
        CARGO_TARGET_DIR=str(repo / "target"),
        CARGO_BUILD_BUILD_DIR=env.get(
            "CARGO_BUILD_BUILD_DIR", str(Path.home() / ".cache/ohm-build")
        ),
        CARGO_UNSTABLE_OHM_PROC_MACRO_TRUST="all",
        CARGO_UNSTABLE_OHM_NATIVE_TOOL_TRUST="all",
    )
    example = repo / "crates/astral-mail-headers/examples/codec_timing.rs"
    assert not example.exists(), "temporary example path must not exist"
    metadata = {
        "worktree": str(repo),
        "revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=repo, text=True
        ).strip(),
        "source_sha256": {path: sha256(repo / path) for path in SOURCE_FILES},
        "builds": [],
    }
    try:
        shutil.copy2(ROOT / "codec-timing.rs", example)
        for allocator in ("system", "jemalloc"):
            for kind in ("corpus", "supplemental"):
                command = ["cargo", "+ohm", "-Zohm-defaults=no"]
                if kind == "corpus":
                    command += [
                        "bench", "--locked", "--bench", "metadata-extraction", "--no-run"
                    ]
                else:
                    command += [
                        "build", "--release", "--locked", "--example", "codec_timing"
                    ]
                if allocator == "jemalloc":
                    command += ["--features", "benchmark-jemalloc"]
                log = ROOT / f"{label}-{allocator}-{kind}-build.log"
                with log.open("w") as output:
                    subprocess.run(
                        command,
                        cwd=repo,
                        env=env,
                        stdout=output,
                        stderr=subprocess.STDOUT,
                        check=True,
                    )
                if kind == "corpus":
                    match = re.search(r"Executable .+ \((.+)\)", log.read_text())
                    assert match, f"benchmark executable missing from {log}"
                    source = Path(match.group(1))
                else:
                    source = repo / "target/release/examples/codec_timing"
                suffix = "-codec" if kind == "supplemental" else ""
                destination = ROOT / "bin" / f"{label}-{allocator}{suffix}"
                shutil.copy2(source, destination)
                check = [str(destination)]
                if kind == "supplemental":
                    check += [str(ROOT / "codec-inputs.json")]
                subprocess.run(check + ["--test"], cwd=repo, check=True)
                metadata["builds"].append(
                    {
                        "allocator": allocator,
                        "kind": kind,
                        "command": command,
                        "binary": destination.name,
                        "binary_sha256": sha256(destination),
                    }
                )
                print("Built", destination.name, flush=True)
    finally:
        example.unlink(missing_ok=True)
    (ROOT / f"{label}-build.json").write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    main()
