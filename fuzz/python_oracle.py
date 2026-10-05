"""Run the persistent conformance oracle with a separate memory ceiling."""

from pathlib import Path
import resource
import runpy
import sys


resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024,) * 2)
mode = sys.argv[1]
script = Path(__file__).resolve().parents[1] / "scripts" / {
    "headers": "generate_conformance.py",
    "decode": "generate_decode_fixtures.py",
}[mode]
sys.argv = [str(script), "--stdin"]
runpy.run_path(str(script), run_name="__main__")
