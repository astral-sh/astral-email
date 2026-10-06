"""Summarize medians: python3 summarize.py [corpus|supplemental]."""

import csv
import json
import math
from pathlib import Path
import statistics
import sys

ROOT = Path(__file__).resolve().parent


def read(path):
    lines = (line for line in path.read_text().splitlines() if not line.startswith("#"))
    return {row["case"]: row for row in csv.DictReader(lines)}


def geomean(values):
    return math.exp(statistics.mean(math.log(value) for value in values))


def summarize(mode):
    field = "astral_mail_headers_ns" if mode == "corpus" else "median_ns"
    expected_count = 29 if mode == "corpus" else 15
    summaries = {}
    for allocator in ("system", "jemalloc"):
        files = {}
        for variant in ("baseline", "candidate"):
            files[variant] = [
                read(ROOT / "results" / f"{mode}-{allocator}-{variant}-{session}.csv")
                for session in (1, 2, 3)
            ]
        names = files["baseline"][0].keys()
        assert len(names) == expected_count
        for sessions in files.values():
            assert all(session.keys() == names for session in sessions)
        control = None
        if mode == "corpus":
            control = read(ROOT / "results" / f"{mode}-{allocator}-control-1.csv")
            assert control.keys() == names

        cases = []
        for name in names:
            row = {"case": name}
            for variant in ("baseline", "candidate"):
                row[f"{variant}_ns"] = statistics.median(
                    float(session[name][field]) for session in files[variant]
                )
            row["change_percent"] = (row["candidate_ns"] / row["baseline_ns"] - 1) * 100
            repeats = [float(session[name][field]) for session in files["baseline"]]
            row["baseline_repeat_range_percent"] = (max(repeats) / min(repeats) - 1) * 100
            if control is not None:
                row["control_change_percent"] = (
                    float(control[name][field]) / row["baseline_ns"] - 1
                ) * 100
            cases.append(row)

        ratios = [row["candidate_ns"] / row["baseline_ns"] for row in cases]
        changes = [row["change_percent"] for row in cases]
        summary = {
            "cases": cases,
            "geomean_change_percent": (geomean(ratios) - 1) * 100,
            "median_change_percent": statistics.median(changes),
            "change_range_percent": [min(changes), max(changes)],
        }
        if control is not None:
            control_changes = [row["control_change_percent"] for row in cases]
            summary["control_geomean_change_percent"] = (
                geomean([1 + change / 100 for change in control_changes]) - 1
            ) * 100
            summary["control_change_range_percent"] = [
                min(control_changes), max(control_changes)
            ]
        summaries[allocator] = summary
        print(mode, allocator, {key: value for key, value in summary.items() if key != "cases"})
        for row in cases:
            print(
                f"{row['case']}: {row['baseline_ns']:.3f} => "
                f"{row['candidate_ns']:.3f} ns ({row['change_percent']:+.2f}%)"
            )

    (ROOT / f"{mode}-summary.json").write_text(json.dumps(summaries, indent=2) + "\n")
    with (ROOT / f"{mode}-summary.csv").open("w", newline="") as output:
        writer = csv.DictWriter(
            output,
            fieldnames=["allocator", *summaries["system"]["cases"][0]],
            lineterminator="\n",
        )
        writer.writeheader()
        for allocator, summary in summaries.items():
            for row in summary["cases"]:
                writer.writerow({"allocator": allocator, **row})


def main():
    modes = sys.argv[1:] or ["corpus", "supplemental"]
    assert all(mode in ("corpus", "supplemental") for mode in modes)
    for mode in modes:
        summarize(mode)


if __name__ == "__main__":
    main()
