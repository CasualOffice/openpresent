#!/usr/bin/env python3
"""`P1X-01`: is Phase 1 closed? Prints docs/06's closing list as MET or NOT MET, with evidence.

    python3 tools/phase1-exit.py          # runs check.py over the corpus, then CLAUDE.md's gates
    python3 tools/phase1-exit.py --quick  # neither; whatever needs them prints NOT MET: not run

Runs only what already exists, and reads the list from docs/06, so an item added there and
unknown here prints NOT MET. Nothing can be met by assertion: what did not run is not met,
and sign-off is the owner's box. LibreOffice is never started (owner instruction 23), and no
500-slide budget or damage corpus run exists yet, so those items stay NOT MET until one does.

`--tracker` and `--roadmap` let a planted failure be checked.
Exit: 0 if every item is met, 1 if not, 2 if the list cannot be read.
"""

import argparse
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import tracker_rows  # noqa: E402

TIERS = ("1A", "1B", "1C", "1D", "1E", "1F", "1G", "1H", "1I", "1J", "1K", "1L", "1M", "1R", "1N")
# Tiers with no `T-` row of their own: the rows that stand for them.
STAND_INS = {"1R": r"REC-0[1-5]", "1N": r"T1N-\d+"}
GATES = (
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
    "cargo test --workspace --all-features --locked",
    "cargo doc --workspace --all-features --no-deps --locked",
    "cargo check --target wasm32-unknown-unknown --workspace --locked",
    "python3 tools/check-repo.py",
    "python3 tools/spec-checklist/generate.py --check",
    "python3 fixtures/tools/generate.py --check",
    "python3 tools/spec-audit/audit.py --check",
    "python3 tools/ooxml-sdk/validate.py fixtures/generated",
)
ENV = {"CARGO_BUILD_JOBS": "2", "RUST_TEST_THREADS": "2", "F4_REQUIRED": "1", "RUSTDOCFLAGS": "-D warnings"}


def closing_list(path):
    lines = pathlib.Path(path).read_text(encoding="utf8").splitlines()
    if "**Phase 1 closes when:**" not in lines:
        return []
    items = []
    for line in lines[lines.index("**Phase 1 closes when:**") + 1:]:
        match = re.match(r"- \[([ xX])\] (.*)", line)
        if not match:
            break
        items.append((match.group(2), match.group(1) != " "))
    return items


def tiers(args):
    """A tier is closed when its row, and every leftover row of that tier, is Done."""
    text = pathlib.Path(args.tracker).read_text(encoding="utf8")
    status = {i: tracker_rows.cells(line)[2] for _, i, line in tracker_rows.rows(text)}
    still_open = []
    for tier in TIERS:
        pattern = STAND_INS[tier] if f"T-{tier}" not in status and tier in STAND_INS else rf"T-{tier}|T{tier}-\d+"
        members = [i for i in status if re.fullmatch(pattern, i)]
        if f"T-{tier}" not in status and tier not in STAND_INS:
            members = []
        unfinished = [i for i in members if status[i] != "Done"]
        if not members or unfinished:
            still_open.append(f"{tier} ({', '.join(unfinished[:2]) or 'no row'})")
    evidence = f"{len(TIERS) - len(still_open)} of {len(TIERS)} closed"
    return not still_open, evidence + ("; open: " + "; ".join(still_open) if still_open else "")


def exports(args):
    """check.py: engine, F4 and the SDK validator. It validates each input, and compares F4
    before and after the forced rewrite. LibreOffice's reopen cannot be shown here."""
    if args.quick:
        return False, "not run (--quick); LibreOffice reopen not run either (FID-04)"
    with tempfile.TemporaryDirectory(prefix="phase1-exit-") as tmp:
        report = pathlib.Path(tmp) / "check.json"
        subprocess.run([sys.executable, "-I", str(ROOT / "tools/real-decks/check.py"), "--report", str(report),
                        str(ROOT / "fixtures/real"), str(ROOT / "fixtures/producers")],
                       capture_output=True, text=True, check=False)
        if not report.is_file():
            return False, "check.py wrote no report"
        results = json.loads(report.read_text(encoding="utf8"))
    bad = [r["deck"] for r in results if not (r["engine_ok"] and r["f4"] == "pass" and r["sdk"] == "pass"
                                              and r["f4_same_after_rewrite"])]
    return False, (f"{len(results) - len(bad)} of {len(results)} committed decks clean, failing: {', '.join(bad[:5]) or 'none'}; "
                   "LibreOffice reopen not run here (FID-04), so this stays NOT MET")


def corpus(args):
    """The provenance table's writer and platform columns, for the producers docs/06 names."""
    text = (ROOT / "fixtures/real/README.md").read_text(encoding="utf8")
    rows = [tracker_rows.cells(ln) for ln in text.splitlines() if re.match(r"\| `[\w.-]+` \|", ln)]
    rows = [r for r in rows if len(r) >= 9]
    counts = {
        "PowerPoint Windows": sum("PowerPoint" in r[7] and "Mac" not in r[7] + r[8] for r in rows),
        "PowerPoint Mac": sum("PowerPoint" in r[7] and "Mac" in r[7] + r[8] for r in rows),
        "Keynote": sum("Keynote" in r[7] for r in rows),
        "Google Slides": sum("Google" in r[7] for r in rows),
    }
    return all(counts.values()), ", ".join(f"{k} {v}" for k, v in counts.items()) + f" of {len(rows)} recorded decks"


CHECKS = (
    (r"^All tiers closed", lambda a, ticked: tiers(a)),
    (r"exported corpus deck", lambda a, ticked: exports(a)),
    (r"^Corpus includes", lambda a, ticked: corpus(a)),
    (r"500-slide", lambda a, ticked: (False, "no 500-slide budget or whole-corpus hostile run exists yet")),
    (r"damage corpus", lambda a, ticked: (False, "no damage corpus run exists yet (REC-*)")),
    (r"^Owner sign-off", lambda a, ticked: (ticked, "the box in docs/06")),
)


def gates(args):
    if args.quick:
        return False, "not run (--quick)"
    env = dict(os.environ, **ENV)
    failed = [g for g in GATES if subprocess.run(g.split(), cwd=ROOT, env=env, capture_output=True, check=False).returncode]
    return not failed, f"failed: {'; '.join(failed)}" if failed else f"all {len(GATES)} pass, with F4_REQUIRED=1"


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--tracker", default=str(ROOT / tracker_rows.TRACKER))
    parser.add_argument("--roadmap", default=str(ROOT / "docs/06-ROADMAP-AND-DELIVERY.md"))
    args = parser.parse_args(argv[1:])
    items = closing_list(args.roadmap)
    if not items:
        print(f"no Phase 1 closing list in {args.roadmap}", file=sys.stderr)
        return 2
    results = []
    for text, ticked in items:
        check = next((fn for pattern, fn in CHECKS if re.search(pattern, text)), None)
        results.append((text, *(check(args, ticked) if check else (False, "this tool has no check for this item"))))
    results.append(("Gates in CLAUDE.md", *gates(args)))
    for text, met, evidence in results:
        print(f"{'MET    ' if met else 'NOT MET'} {text}\n        {evidence}")
    closed = sum(r[1] for r in results)
    print(f"{closed} of {len(results)} met.")
    return 0 if closed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
