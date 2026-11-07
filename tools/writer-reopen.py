#!/usr/bin/env python3
"""T1N-04: what the writer produced, reopened by what we do not control.

`crates/casual-slide-export/tests/writer_corpus.rs` writes each result to
<dir>/<variant>/<deck>, with <deck>.clones naming each clone's source.

    python3 tools/writer-reopen.py f4 <dir>     # each result against its input
    python3 tools/writer-reopen.py sdk <dir>    # F-006, the same way
    python3 tools/writer-reopen.py sdk-rr       # fixtures/rr/good, each against its base

A part that failed in the input may fail in a result, under its own name or its clone's;
nothing else may. fixtures/hostile is refused here outright. This tool does not run
LibreOffice: the owner stopped that while it was crashing on their machine.

Exit: 0 if nothing new failed, 1 if something did, 2 if it could not run.
"""
import json
import os
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[1]
FIXTURES = REPO / "fixtures"
INPUT_DIRS = ("generated", "producers", "real")
BAD = {"invalid", "malformed", "refused", "must-understand", "schema-error"}


def results(root):
    root = pathlib.Path(root)
    if "hostile" in root.parts:
        raise SystemExit("fixtures/hostile is never reopened")
    for variant in sorted(p for p in root.iterdir() if p.is_dir()):
        for deck in sorted(variant.iterdir()):
            if deck.suffix != ".clones":
                yield variant.name, deck


def input_of(deck):
    for folder in INPUT_DIRS:
        path = FIXTURES / folder / deck.name
        if path.exists():
            return path
    raise SystemExit(f"{deck.name}: no input deck")


def clones_of(deck):
    path = deck.with_name(deck.name + ".clones")
    if not path.exists():
        return {}
    return dict(line.split("\t") for line in path.read_text().splitlines() if line)


def f4(paths):
    """Part statuses by deck, keyed by path. Exit 1 just means something failed."""
    tool = REPO / "tools/f4/validate.py"
    out = subprocess.run([sys.executable, "-I", str(tool), "--json", *map(str, paths)],
                         capture_output=True, text=True, check=False)
    if out.returncode not in (0, 1):
        raise SystemExit(f"F4 could not run:\n{out.stderr}")
    return json.loads(out.stdout)


def run_f4(root):
    rows = list(results(root))
    inputs = sorted({input_of(deck) for _, deck in rows})
    bad_before = {}
    for report in f4(inputs):
        failed = {p["part"] for p in report["parts"] if p["status"] in BAD}
        bad_before[report["deck"]] = failed
    new = []
    for variant in sorted({v for v, _ in rows}):
        decks = [deck for v, deck in rows if v == variant]
        for report, deck in zip(f4(decks), decks):
            if report["deck"] != deck.name:
                raise SystemExit(f"F4 reports out of order: {report['deck']} against {deck.name}")
            clones = clones_of(deck)
            for part in report["parts"]:
                if part["status"] in BAD:
                    origin = clones.get(part["part"], part["part"])
                    if origin not in bad_before.get(deck.name, set()):
                        new.append(f"{variant}/{deck.name}: {part['part']} {part['status']}: "
                                   f"{part['errors'][:2]}")
    print(f"F4: {len(rows)} results against {len(inputs)} inputs, {len(new)} new failures")
    for line in new:
        print(" ", line)
    return 1 if new else 0


def sdk_report(paths):
    tool = REPO / "tools/ooxml-sdk/validate.py"
    env = dict(os.environ, OOXML_SDK_REQUIRED="1")
    out = subprocess.run([sys.executable, "-I", str(tool), "--json", "--no-known", *map(str, paths)],
                         capture_output=True, text=True, check=False, env=env)
    if out.returncode not in (0, 1):
        raise SystemExit(f"the SDK validator could not run:\n{out.stderr}\n{out.stdout}")
    return json.loads(out.stdout)


def findings_by_deck(report):
    """{deck: {(part, rule)}} from the SDK tool's JSON."""
    found = {}
    for row in report["decks"]:
        rows = found.setdefault(row["deck"], set())
        if row["status"] == "refused":
            rows.add(("*", f"refused:{row['reason']}"))
        for finding in row["errors"]:
            rows.add((finding["part"], finding["id"]))
    return found


def run_sdk(root):
    rows = list(results(root))
    inputs = sorted({input_of(deck) for _, deck in rows})
    before = findings_by_deck(sdk_report(inputs))
    new = []
    for variant in sorted({v for v, _ in rows}):
        decks = [deck for v, deck in rows if v == variant]
        after = findings_by_deck(sdk_report(decks))
        for deck in decks:
            clones = clones_of(deck)
            for part, rule in sorted(after.get(deck.name, ())):
                origin = clones.get(part.lstrip("/"), part.lstrip("/"))
                known = {(p.lstrip("/"), r) for p, r in before.get(deck.name, ())}
                if (origin, rule) not in known:
                    new.append(f"{variant}/{deck.name}: {part} {rule}")
    print(f"SDK: {len(rows)} results against {len(inputs)} inputs, {len(new)} new findings")
    for line in new:
        print(" ", line)
    return 1 if new else 0


def run_sdk_rr(_root=None):
    rows = [ln.split("\t") for ln in (FIXTURES / "rr/manifest.tsv").read_text().splitlines()[1:]]
    good = [FIXTURES / "rr" / row[3] for row in rows]
    bases = sorted({FIXTURES / row[2] for row in rows})
    before = findings_by_deck(sdk_report(bases))
    after = findings_by_deck(sdk_report(good))
    new = []
    for row in rows:
        known = before.get(pathlib.Path(row[2]).name, set())
        for finding in sorted(after.get(pathlib.Path(row[3]).name, ())):
            if finding not in known:
                new.append(f"{row[0]}: {finding}")
    print(f"SDK: {len(good)} good decks against {len(bases)} bases, {len(new)} new findings")
    for line in new:
        print(" ", line)
    return 1 if new else 0


def main(argv):
    if argv == ["sdk-rr"]:
        return run_sdk_rr()
    if len(argv) != 2 or argv[0] not in ("f4", "sdk"):
        print(__doc__)
        return 2
    return {"f4": run_f4, "sdk": run_sdk}[argv[0]](argv[1])


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
