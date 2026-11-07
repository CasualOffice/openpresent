#!/usr/bin/env python3
"""`P1X-01`: is Phase 1 closed? Prints docs/06's closing list as MET or NOT MET, with evidence.

    python3 tools/phase1-exit.py          # reads the evidence record, then runs CLAUDE.md's gates
    python3 tools/phase1-exit.py --quick  # without the gates, which then print NOT MET: not run

Runs only what already exists, and reads the list from docs/06, so an item added there and
unknown here prints NOT MET. Nothing can be met by assertion: what did not run is not met,
and sign-off is the owner's box.

The 500-slide budget, the hostile run, the damage corpus and the exports through F4, the SDK
and LibreOffice are too slow to run here. `tools/phase1-evidence.py` runs each and records it in
tools/phase1-evidence.json; this reads that record. An item is MET only if its record shows no
failure and was made from the fixtures as they are now. The commit it was made at is printed:
an engine change since then is a reason to run it again before sign-off.

`--tracker`, `--roadmap` and `--evidence` let a planted failure be checked.
Exit: 0 if every item is met, 1 if not, 2 if the list cannot be read.
"""

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys

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


def evidence(args, key):
    path = pathlib.Path(args.evidence)
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf8")).get(key)


def digest(paths):
    """As phase1-evidence.py makes it: names and bytes, so a changed corpus is seen."""
    fixtures = ROOT / "fixtures"
    h = hashlib.sha256()
    for p in sorted(paths, key=lambda p: str(p.relative_to(fixtures))):
        h.update(str(p.relative_to(fixtures)).encode() + b"\0" + hashlib.sha256(p.read_bytes()).digest())
    return h.hexdigest()


def when(record):
    return f"run {record.get('date')} at {str(record.get('commit'))[:9]}, load {record.get('load_before')}"


DECK_SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}


def libreoffice_own(failure):
    """A LibreOffice failure that its unedited input shows too: `LibreOffice edited/x: crash 134,
    unedited input crash`."""
    if not failure.startswith("LibreOffice "):
        return False
    _, _, rest = failure.partition(": ")
    ours, _, theirs = rest.partition(", unedited input ")
    return bool(theirs) and ours.split()[0] == theirs.split()[0]


def exports(args):
    """Every corpus deck written out edited, and the save-as grid, through F4, the SDK and LibreOffice."""
    record = evidence(args, "export")
    if not record:
        return False, "no export run recorded (tools/phase1-evidence.py export)"
    fixtures = ROOT / "fixtures"
    decks = [p for d in ("real", "generated") for p in sorted((fixtures / d).iterdir()) if p.suffix in DECK_SUFFIXES]
    saveas = sorted(p for p in (fixtures / "rr/sa").iterdir() if p.suffix != ".tsv")
    if record.get("inputs_sha256") != digest(decks + saveas):
        return False, f"the record is for other decks than fixtures/real, generated and rr/sa now hold; {when(record)}"
    # Owner instruction 40: LibreOffice failing on a deck whose unedited input fails the same way
    # is LibreOffice's own fault, not ours, and does not hold this item back.
    theirs = [f for f in record.get("failures", []) if libreoffice_own(f)]
    failures = [f for f in record.get("failures", []) if f not in theirs]
    met = not failures and record.get("written") == len(decks) \
        and record.get("libreoffice_checked") == len(decks) + sum(p.suffix in DECK_SUFFIXES for p in saveas)
    text = (f"{record.get('written')} decks written edited and {len(saveas)} save-as files; F4 and SDK on "
            f"{record.get('f4_and_sdk_checked')}, LibreOffice ({record.get('libreoffice_image')}) on "
            f"{record.get('libreoffice_checked')}: {record.get('libreoffice')}; {len(failures)} failures")
    if failures:
        text += ": " + "; ".join(failures[:4]) + ("; ..." if len(failures) > 4 else "")
    if theirs:
        text += f"; {len(theirs)} more are LibreOffice failing on the unedited input too (instruction 40)"
    return met, f"{text}; {when(record)}"


def budget_and_hostile(args):
    """The 500-slide deck within its budget, and every hostile file opened and edited, or refused,
    with no panic and none past its bound."""
    budget, hostile = evidence(args, "budget"), evidence(args, "hostile")
    notes, met = [], True
    deck = ROOT / "fixtures/generated/five-hundred-slides.pptx"
    if not budget:
        met = False
        notes.append("no 500-slide run recorded")
    else:
        measured, limit = budget.get("measured_ms", {}), budget.get("budget_ms", {})
        over = [k for k in ("open", "unedited_save", "edit") if k not in measured or measured[k] > limit.get(k, -1)]
        fresh = budget.get("deck_sha256") == hashlib.sha256(deck.read_bytes()).hexdigest()
        met &= budget.get("ok") is True and not over and fresh
        notes.append(f"500 slides, {'optimized' if budget.get('optimized') else 'unoptimized'}, ms measured/budget: "
                     + ", ".join(f"{k} {measured.get(k)}/{limit.get(k)}" for k in ("open", "unedited_save", "edit"))
                     + (f", OVER: {', '.join(over)}" if over else "") + ("" if fresh else ", for another deck")
                     + f" ({when(budget)})")
    files = sorted(p for p in (ROOT / "fixtures/hostile").iterdir() if p.is_file())
    if not hostile:
        met = False
        notes.append("no hostile run recorded")
    else:
        fresh = hostile.get("fixtures_sha256") == digest(files)
        met &= hostile.get("ok") is True and hostile.get("broken") == 0 and hostile.get("run") == len(files) and fresh
        notes.append(f"hostile: {hostile.get('run')} of {len(files)} files, {hostile.get('edited')} edited, "
                     f"{hostile.get('refused')} refused by code, {hostile.get('broken')} crashed or hung, slowest "
                     f"{hostile.get('slowest_ms')} ms of {hostile.get('bound_ms')}" + ("" if fresh else ", for other files")
                     + f" ({when(hostile)})")
    return met, "; ".join(notes)


def damage(args):
    record = evidence(args, "damage")
    if not record:
        return False, "no damage corpus run recorded (tools/phase1-evidence.py damage)"
    decks = sorted(p for p in (ROOT / "fixtures/real").iterdir() if p.suffix == ".pptx")
    fresh = record.get("decks_sha256") == digest(decks)
    met = record.get("ok") is True and record.get("exit") == 0 and fresh
    text = "; ".join(str(record.get(k)) for k in ("corpus", "opened", "f4") if record.get(k))
    if record.get("failure"):
        text += "; FAILED: " + record["failure"][:300].replace("\n", " | ")
    return met, f"DAMAGE_FULL=1: {text}" + ("" if fresh else "; for other decks than fixtures/real holds") + f" ({when(record)})"


def corpus(args):
    """The provenance table's writer and platform columns, for the producers docs/06 names."""
    text = (ROOT / "fixtures/real/README.md").read_text(encoding="utf8")
    rows = [tracker_rows.cells(ln) for ln in text.splitlines() if re.match(r"\| `[\w.-]+` \|", ln)]
    rows = [r for r in rows if len(r) >= 9]
    counts = {
        "PowerPoint Windows": sum("PowerPoint" in r[7] and "Mac" not in r[7] + r[8] for r in rows),
        "PowerPoint Mac": sum("PowerPoint" in r[7] and "Mac" in r[7] + r[8] for r in rows),
        "Google Slides": sum("Google" in r[7] for r in rows),
    }
    return all(counts.values()), ", ".join(f"{k} {v}" for k, v in counts.items()) + f" of {len(rows)} recorded decks"


CHECKS = (
    (r"^All tiers closed", lambda a, ticked: tiers(a)),
    (r"exported corpus deck", lambda a, ticked: exports(a)),
    (r"^Corpus includes", lambda a, ticked: corpus(a)),
    (r"500-slide", lambda a, ticked: budget_and_hostile(a)),
    (r"damage corpus", lambda a, ticked: damage(a)),
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
    parser.add_argument("--evidence", default=str(ROOT / "tools/phase1-evidence.json"))
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
