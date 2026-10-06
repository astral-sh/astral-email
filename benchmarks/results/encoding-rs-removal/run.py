"""Collect interleaved sessions: python3 run.py corpus|supplemental."""

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def run(mode, allocator, variant, session):
    name = f"{mode}-{allocator}-{variant}-{session}"
    implementation = "baseline" if variant == "control" else variant
    suffix = "-codec" if mode == "supplemental" else ""
    executable = ROOT / "bin" / f"{implementation}-{allocator}{suffix}"
    command = ["taskset", "-c", "0", str(executable)]
    if mode == "supplemental":
        command += [str(ROOT / "codec-inputs.json")]
    else:
        command += [
            "--bench", "--samples", "31", "--sample-ms", "20", "--warmup-ms", "100"
        ]
    started = timestamp()
    print("Starting", name, started, flush=True)
    csv_path = ROOT / "results" / f"{name}.csv"
    with csv_path.open("w") as output:
        subprocess.run(command, check=True, stdout=output, cwd=ROOT)
    record = {
        "name": name,
        "command": command,
        "started_at": started,
        "finished_at": timestamp(),
        "binary_sha256": sha256(executable),
        "csv_sha256": sha256(csv_path),
    }
    print("Finished", name, record["finished_at"], flush=True)
    return record


def main():
    mode = sys.argv[1]
    assert mode in ("corpus", "supplemental")
    (ROOT / "results").mkdir(exist_ok=True)
    records = []
    metadata = ROOT / f"{mode}-runs.json"

    def record(allocator, variant, session):
        records.append(run(mode, allocator, variant, session))
        metadata.write_text(json.dumps(records, indent=2) + "\n")

    for session in range(1, 4):
        allocators = ("system", "jemalloc") if session % 2 else ("jemalloc", "system")
        variants = ("baseline", "candidate") if session % 2 else ("candidate", "baseline")
        for allocator in allocators:
            for variant in variants:
                record(allocator, variant, session)
    if mode == "corpus":
        for allocator in ("system", "jemalloc"):
            record(allocator, "control", 1)


if __name__ == "__main__":
    main()
