"""Run the persistent conformance oracle with a separate memory ceiling."""

from pathlib import Path
import resource
import runpy
import sys


resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024,) * 2)
script = Path(__file__).resolve().parents[1] / "scripts/generate_conformance.py"
sys.argv = [str(script), "--stdin"]
runpy.run_path(str(script), run_name="__main__")
