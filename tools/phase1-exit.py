#!/usr/bin/env python3
"""`P1X-01`: is Phase 1 closed? Runs the evidence and prints docs/06's closing list.

    python3 -I tools/phase1-exit.py [--local DIR] [--libreoffice FILE] [--no-gates] [--json FILE]

Reads the closing list from docs/06 at run time, so an item added there and unknown here is
reported NOT MET, not skipped. Prints each item as MET or NOT MET with its evidence, and
the phase percentage from the checklist. **Nothing here can mark an item met by assertion:**
a check that did not run is not met, and the owner's sign-off is only ever the owner's.

  Tiers       every tier's row in docs/14 is Done, and every leftover row of that tier too.
              1N and 1R have no T- row yet, so their T1N-* and REC-01..05 rows stand in.
  Exports     check.py over the corpus: each forced rewrite passes F4 and Microsoft's validator.
              LibreOffice is never started here (owner instruction 23). Its reopen comes from
              --libreoffice, a TSV of `sha256<TAB>ok` from a Linux container run over those
              exports (libreoffice_reopen.py), and must cover every one of them.
  Corpus      census.py: the four named producers, recorded and at target; all of CORP-01
              is shown as evidence, and blocks through the gates line below.
  Budget      budget.py and hostile_run.py.
  Damage      damage.py.
  Sign-off    the box in docs/06 ticked by the owner.

The gates are CLAUDE.md's list with F4_REQUIRED=1, as one more line, which must also pass.
`--no-gates` skips them while working, and they then print as NOT MET: not run.

Run from a checkout. `--tracker`, `--roadmap`, `--checklist`, `--targets`, `--probe` and
`--fixtures` exist so a test can plant a failure. Exit: 0 if every item is met, 1 if any is
not, 2 if the run could not start.
"""

import argparse
import csv
import hashlib
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

TOOLS = ROOT / "tools/real-decks"
NAMED_PRODUCERS = ("PowerPoint Windows", "PowerPoint Mac", "Keynote", "Google Slides")
TIERS = ("1A", "1B", "1C", "1D", "1E", "1F", "1G", "1H", "1I", "1J", "1K", "1L", "1M", "1R", "1N")
# A tier with no `T-` row of its own: the rows that stand for it.
STAND_INS = {"1R": re.compile(r"REC-0[1-5]$"), "1N": re.compile(r"T1N-\d+$")}
GATES = (
    ("fmt", ["cargo", "fmt", "--all", "--", "--check"]),
    ("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"]),
    ("test", ["cargo", "test", "--workspace", "--all-features", "--locked"]),
    ("doc", ["cargo", "doc", "--workspace", "--all-features", "--no-deps", "--locked"]),
    ("wasm", ["cargo", "check", "--target", "wasm32-unknown-unknown", "--workspace", "--locked"]),
    ("repo", [sys.executable, "tools/check-repo.py"]),
    ("checklist", [sys.executable, "tools/spec-checklist/generate.py", "--check"]),
    ("fixtures", [sys.executable, "fixtures/tools/generate.py", "--check"]),
    ("audit", [sys.executable, "tools/spec-audit/audit.py", "--check"]),
    ("sdk", [sys.executable, "tools/ooxml-sdk/validate.py", "fixtures/generated"]),
)
CARGO_ENV = {"CARGO_BUILD_JOBS": "2", "RUST_TEST_THREADS": "2", "F4_REQUIRED": "1", "RUSTDOCFLAGS": "-D warnings",
             "OOXML_SDK_REQUIRED": "1"}


def run_json(command, report):
    """Run a tool writing a JSON report: `(exit, report or None, last line of output)`."""
    done = subprocess.run(command, capture_output=True, text=True, check=False)
    lines = [ln for ln in done.stdout.splitlines() if ln.strip()]
    parsed = None
    path = pathlib.Path(report)
    if path.is_file():
        try:
            parsed = json.loads(path.read_text(encoding="utf8"))
        except json.JSONDecodeError:
            parsed = None
    return done.returncode, parsed, (lines[-1] if lines else done.stderr.strip()[-300:])


def closing_list(roadmap):
    """docs/06's Phase 1 closing items: `(text, ticked)`. Empty if the list cannot be found."""
    lines = pathlib.Path(roadmap).read_text(encoding="utf8").splitlines()
    try:
        start = lines.index("**Phase 1 closes when:**") + 1
    except ValueError:
        return []
    items = []
    for line in lines[start:]:
        match = re.match(r"- \[([ xX])\] (.*)", line)
        if not match:
            break
        items.append((match.group(2), match.group(1) != " "))
    return items


def tiers(tracker):
    """Each tier's state from the tracker: `(met, evidence)`."""
    rows = list(tracker_rows.rows(pathlib.Path(tracker).read_text(encoding="utf8")))
    status = {row_id: tracker_rows.cells(line)[2] for _, row_id, line in rows}
    open_tiers = []
    for tier in TIERS:
        head = f"T-{tier}"
        leftovers = [i for i in status if re.fullmatch(rf"T{tier}-\d+", i)]
        if head in status:
            members = [head] + leftovers
        elif tier in STAND_INS:
            members = [i for i in status if STAND_INS[tier].match(i)]
        else:
            members = []
        if not members:
            open_tiers.append(f"{tier} (no row)")
            continue
        unfinished = [i for i in members if status[i] != "Done"]
        if unfinished:
            open_tiers.append(f"{tier} ({', '.join(unfinished[:3])}{'...' if len(unfinished) > 3 else ''})")
    closed = len(TIERS) - len(open_tiers)
    evidence = f"{closed} of {len(TIERS)} tiers closed"
    if open_tiers:
        evidence += "; open: " + "; ".join(open_tiers)
    return not open_tiers, evidence


def percent(checklist):
    """Phase 1's share: checklist rows with model, import and export each marked y or r."""
    csv.field_size_limit(1 << 24)
    with open(checklist, encoding="utf8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE))
    done = sum(all(r[c] in ("y", "r") for c in ("M", "I", "E")) for r in rows)
    return done, len(rows)


def exports(args, tmp):
    """The forced rewrite of every corpus deck, through F4 and the SDK validator."""
    report = tmp / "check.json"
    keep = tmp / "exports"
    inputs = [str(args.fixtures / "real"), str(args.fixtures / "producers")]
    if args.local:
        inputs.append(args.local)
    code, results, last = run_json([sys.executable, "-I", str(TOOLS / "check.py"), "--report", str(report),
                                    "--keep-exports", str(keep), *inputs], report)
    if not results:
        return False, f"check.py did not report (exit {code}): {last}"
    bad = [r["deck"] for r in results
           if not (r.get("engine_ok") and r.get("f4_after") == "pass" and r.get("sdk_after") in ("PASS", "KNOWN"))]
    known = sum(r.get("sdk_after") == "KNOWN" for r in results)
    evidence = f"{len(results) - len(bad)} of {len(results)} exports pass the engine, F4 and the SDK validator ({known} only as recorded, for PV-01)"
    if bad:
        evidence += f"; failing: {', '.join(bad[:6])}{'...' if len(bad) > 6 else ''}"
    shas = {hashlib.sha256(p.read_bytes()).hexdigest() for p in keep.iterdir()} if keep.is_dir() else set()
    reopen_ok, reopen = libreoffice(args.libreoffice, shas)
    return (not bad and reopen_ok), f"{evidence}. {reopen}. PowerPoint itself is PV-01"


def libreoffice(path, shas):
    if not path:
        return False, "LibreOffice reopen not run: it is never started here (FID-04); needs --libreoffice from a container"
    try:
        lines = pathlib.Path(path).read_text(encoding="utf8").splitlines()
    except OSError as err:
        return False, f"LibreOffice evidence unreadable: {err.strerror}"
    good = {cols[0] for cols in (ln.split("\t") for ln in lines) if len(cols) >= 2 and cols[1] == "ok"}
    missing = shas - good
    if not shas or missing:
        return False, f"LibreOffice reopen covers {len(shas) - len(missing)} of {len(shas)} exports"
    return True, f"LibreOffice reopened all {len(shas)} exports, by SHA-256"


def corpus(args, tmp):
    report = tmp / "census.json"
    command = [sys.executable, "-I", str(TOOLS / "census.py"), "--json", str(report), "--targets", args.targets]
    if args.local:
        command += ["--local", args.local]
    code, data, last = run_json(command, report)
    if not data:
        return False, f"census.py did not report (exit {code}): {last}"
    results = data["results"]
    named = [r for r in results if r["axis"] == "producer" and r["value"] in NAMED_PRODUCERS]
    under = [r for r in results if not r["met"]]
    shown = "; ".join(f"{r['value']} {r['have']} of {r['min']}" for r in named)
    missing = [p for p in NAMED_PRODUCERS if p not in {r["value"] for r in named}]
    evidence = f"{shown}. CORP-01: {len(results) - len(under)} of {len(results)} targets met"
    if under:
        evidence += "; under: " + ", ".join(f"{r['axis']} {r['value']}" for r in under[:8]) + ("..." if len(under) > 8 else "")
    if missing:
        evidence += f"; no target for {', '.join(missing)}"
    if data.get("uncounted"):
        evidence += f"; {len(data['uncounted'])} decks without provenance not counted"
    return (code == 0 and not missing), evidence


def budget_and_hostile(args, tmp):
    budget_report, hostile_report = tmp / "budget.json", tmp / "hostile.json"
    probe = ["--probe", args.probe] if args.probe else []
    code_b, budget, last_b = run_json([sys.executable, "-I", str(TOOLS / "budget.py"), "--json", str(budget_report),
                                       "--validate", *probe], budget_report)
    local = ["--local", args.local] if args.local else []
    code_h, hostile, last_h = run_json([sys.executable, "-I", str(TOOLS / "hostile_run.py"), "--json", str(hostile_report),
                                        "--fixtures", str(args.fixtures), *probe, *local], hostile_report)
    parts = []
    if budget:
        stages = ", ".join(f"{k} {v['best_ms']} ms {v['peak_mib']:.0f} MiB" for k, v in budget["stages"].items())
        parts.append(f"500 slides: {stages or 'no stage ran'}; {'; '.join(budget['failures']) or 'within budget'}; {budget['load']}")
    else:
        parts.append(f"budget.py did not report (exit {code_b}): {last_b}")
    if hostile:
        parts.append(f"hostile: {hostile['decks']} decks ({hostile['hostile']} hostile), {len(hostile['failures'])} crash or hang"
                     + (f": {hostile['failures'][0]['deck']}: {hostile['failures'][0]['why']}" if hostile["failures"] else "")
                     + f"; {hostile['hostile_refused']} hostile decks still refused")
    else:
        parts.append(f"hostile_run.py did not report (exit {code_h}): {last_h}")
    return (code_b == 0 and code_h == 0 and bool(budget) and bool(hostile)), ". ".join(parts)


def damage(args, tmp):
    report = tmp / "damage.json"
    probe = ["--probe", args.probe] if args.probe else []
    code, data, last = run_json([sys.executable, "-I", str(TOOLS / "damage.py"), "--json", str(report),
                                 "--fixtures", str(args.fixtures), *probe], report)
    if not data:
        return False, f"damage.py did not report (exit {code}): {last}"
    evidence = f"{data['cases']} damaged decks: {data['verdicts']}"
    if data["codes"]:
        evidence += f"; refusals against docs/29: {data['codes']}"
    return code == 0, evidence


def gates(skip):
    if skip:
        return False, "not run (--no-gates)"
    environment = dict(os.environ, **CARGO_ENV)
    failed = []
    for name, command in GATES:
        extra = {"RUSTDOCFLAGS": "-D warnings"} if name == "doc" else {}
        done = subprocess.run(command, cwd=ROOT, env=dict(environment, **extra), capture_output=True, text=True, check=False)
        if done.returncode != 0:
            failed.append(name)
    if failed:
        return False, f"failed: {', '.join(failed)}"
    return True, f"all {len(GATES)} pass, with F4_REQUIRED=1"


# How each docs/06 item is checked. The first pattern matching an item's text wins.
CHECKS = (
    (r"^All tiers closed", lambda a, t, ticked: tiers(a.tracker)),
    (r"exported corpus deck", lambda a, t, ticked: exports(a, t)),
    (r"^Corpus includes", lambda a, t, ticked: corpus(a, t)),
    (r"500-slide", lambda a, t, ticked: budget_and_hostile(a, t)),
    (r"damage corpus", lambda a, t, ticked: damage(a, t)),
    (r"^Owner sign-off", lambda a, t, ticked: (ticked, "ticked by the owner in docs/06" if ticked else "the owner has not signed")),
)


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--local", help="directory of fetched decks, as fetch.py fills it")
    parser.add_argument("--libreoffice", help="TSV from a container: sha256, ok")
    parser.add_argument("--no-gates", action="store_true")
    parser.add_argument("--json")
    parser.add_argument("--only", help="a word: run only items whose text matches; others print NOT MET: not run")
    parser.add_argument("--tracker", default=str(ROOT / tracker_rows.TRACKER))
    parser.add_argument("--roadmap", default=str(ROOT / "docs/06-ROADMAP-AND-DELIVERY.md"))
    parser.add_argument("--checklist", default=str(ROOT / "docs/spec/checklist.tsv"))
    parser.add_argument("--targets", default=str(TOOLS / "targets.tsv"))
    parser.add_argument("--probe")
    parser.add_argument("--fixtures", type=pathlib.Path, default=ROOT / "fixtures")
    args = parser.parse_args(argv[1:])

    try:
        items = closing_list(args.roadmap)
    except OSError as err:
        print(f"cannot read the closing list: {err}", file=sys.stderr)
        return 2
    if not items:
        print(f"no Phase 1 closing list in {args.roadmap}", file=sys.stderr)
        return 2

    results = []
    with tempfile.TemporaryDirectory(prefix="phase1-exit-") as tmp:
        for text, ticked in items:
            check = next((fn for pattern, fn in CHECKS if re.search(pattern, text)), None)
            if args.only and not re.search(args.only, text):
                met, evidence = False, "not run (--only)"
            elif check is None:
                met, evidence = False, "this tool has no check for this item"
            else:
                met, evidence = check(args, pathlib.Path(tmp), ticked)
            results.append({"item": text, "met": met, "evidence": evidence})
            print(f"{'MET    ' if met else 'NOT MET'} {text}\n        {evidence}", flush=True)
    met, evidence = (False, "not run (--only)") if args.only else gates(args.no_gates)
    results.append({"item": "Gates in CLAUDE.md", "met": met, "evidence": evidence})
    print(f"{'MET    ' if met else 'NOT MET'} Gates in CLAUDE.md\n        {evidence}")

    done, total = percent(args.checklist)
    print(f"Phase 1: {done} of {total} checklist rows ({100 * done / total:.1f}%) have model, import and export marked.")
    closed = sum(r["met"] for r in results)
    print(f"{closed} of {len(results)} items met. Phase 1 is {'ready for the owner' if closed == len(results) else 'not closed'}.")
    if args.json:
        pathlib.Path(args.json).write_text(json.dumps(results, indent=1), encoding="utf8")
    return 0 if closed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
