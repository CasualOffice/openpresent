"""`P1X-01`: what the exit tools share: building the probe, and running one bounded process.

Decks are untrusted, so each operation runs in its own process, which a time bound can
kill, whose exit status tells a crash from an answer, and whose peak memory `wait4` reports
for that process alone. RUSAGE_CHILDREN would report the largest child so far instead.
POSIX only: the exit run is a Linux or macOS tool.
"""

import json
import os
import pathlib
import signal
import subprocess
import sys
import tempfile
import threading
import time

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROBE = ROOT / "target/real-decks/release/exit_probe"
DECK_SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}


def build_probe():
    """Build the release probe. The numbers mean nothing from a debug build."""
    done = subprocess.run(
        ["cargo", "build", "--release", "--locked", "--manifest-path", str(HERE / "Cargo.toml"),
         "--target-dir", str(ROOT / "target/real-decks")],
        capture_output=True, text=True, check=False,
    )
    if done.returncode != 0:
        raise SystemExit(f"could not build the probe:\n{done.stderr[-2000:]}")
    return PROBE


def bounded(command, seconds):
    """Run a command with a time bound. Returns its status, output, time and peak memory.

    `status` is `exit` with `code`, `signal` with `signal`, or `timeout`.
    """
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        start = time.monotonic()
        process = subprocess.Popen(command, stdout=out, stderr=err, stdin=subprocess.DEVNULL, start_new_session=True)
        fired = threading.Event()

        def kill():
            fired.set()
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass

        timer = threading.Timer(seconds, kill)
        timer.start()
        try:
            _, status, usage = os.wait4(process.pid, 0)
        finally:
            timer.cancel()
        process.returncode = 0  # already reaped; stops Popen waiting on it again
        elapsed = time.monotonic() - start
        out.seek(0)
        err.seek(0)
        stdout, stderr = out.read(), err.read()
    # ru_maxrss is bytes on macOS and KiB on Linux.
    peak = usage.ru_maxrss if sys.platform == "darwin" else usage.ru_maxrss * 1024
    result = {"seconds": round(elapsed, 3), "peak_bytes": peak,
              "stdout": stdout.decode("utf8", "replace"), "stderr": stderr.decode("utf8", "replace")[-400:]}
    if fired.is_set():
        result["status"] = "timeout"
    elif os.WIFSIGNALED(status):
        result.update(status="signal", signal=os.WTERMSIG(status))
    else:
        result.update(status="exit", code=os.WEXITSTATUS(status))
    return result


def answer(result):
    """The probe's JSON line, or None if the process crashed, hung or wrote something else."""
    if result["status"] != "exit" or result.get("code") != 0:
        return None
    try:
        line = json.loads(result["stdout"].strip())
    except json.JSONDecodeError:
        return None
    return line if isinstance(line, dict) and "outcome" in line else None


def describe_crash(result):
    """A crash in a few words, without output from the deck."""
    if result["status"] == "timeout":
        return f"hang: killed after {result['seconds']} s"
    if result["status"] == "signal":
        return f"crash: signal {result['signal']}"
    if result.get("code") != 0:
        return f"crash: exit {result.get('code')}: {result['stderr'].strip()[:160]}"
    return "no answer: output was not the probe's JSON line"


def load_note():
    """Whether timings are distorted, as a sentence and a flag. A budget measured on a
    loaded machine says so (owner's instruction, docs/04 rule 22)."""
    one_minute = os.getloadavg()[0]
    cores = os.cpu_count() or 1
    loaded = one_minute > cores
    word = "LOADED, so these times are not a measure of the code" if loaded else "quiet"
    return f"machine {word}: load {one_minute:.1f} on {cores} cores", loaded


def decks_under(path):
    path = pathlib.Path(path)
    if path.is_dir():
        return sorted(p for p in path.rglob("*") if p.suffix.lower() in DECK_SUFFIXES and p.is_file())
    return [path]
