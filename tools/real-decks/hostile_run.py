#!/usr/bin/env python3
"""`P1X-01`: every hostile fixture through the engine: no crash, no hang, bounded memory.

    python3 -I tools/real-decks/hostile_run.py [--json FILE] [--seconds N] [--memory-mb N]
                                               [--probe PATH] [--local DIR] [--fixtures DIR]

Each deck runs alone in its own process: open, import, every slide part, unedited save.
fixtures/hostile is every row the manifest lists, so a fixture missing from disk fails the
run instead of shrinking it, and every other file there runs too. fixtures/rr/seeded and the
ordinary decks follow, since a crash on those is as much a defect.

A refusal is not a crash and does not fail this run. It is counted, by code, because under
docs/29 most hostile fixtures should come to open; `damage.py` judges refusals.

Bounds: 60 s and 2 GiB per deck, far over what any fixture needs when measured (see the
evidence line), so that a slow machine is not reported as a hang. A hang is what these catch.

Exit: 0 if none crashed or hung, 1 if one did, 2 if the run could not start.
"""

import argparse
import collections
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import exitlib  # noqa: E402

SECONDS = 60
MEMORY_MB = 2048


def hostile_files(fixtures):
    """Every file in fixtures/hostile, and every manifest row for it. Missing ones returned apart."""
    folder = fixtures / "hostile"
    on_disk = sorted(p for p in folder.iterdir() if p.is_file()) if folder.is_dir() else []
    manifest = fixtures / "manifest.tsv"
    listed = []
    if manifest.is_file():
        for line in manifest.read_text(encoding="utf8").splitlines()[1:]:
            rel = line.split("\t", 1)[0]
            if rel.startswith("hostile/"):
                listed.append(fixtures / rel)
    missing = [p.name for p in listed if not p.is_file()]
    return on_disk, missing


def other_files(fixtures, local):
    found = []
    for name in ("rr/seeded", "generated", "producers", "real"):
        folder = fixtures / name
        if folder.is_dir():
            found += exitlib.decks_under(folder)
    if local:
        found += exitlib.decks_under(local)
    return found


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--json")
    parser.add_argument("--seconds", type=float, default=SECONDS)
    parser.add_argument("--memory-mb", type=int, default=MEMORY_MB)
    parser.add_argument("--probe")
    parser.add_argument("--local")
    parser.add_argument("--fixtures", default=str(exitlib.ROOT / "fixtures"))
    args = parser.parse_args(argv[1:])

    fixtures = pathlib.Path(args.fixtures)
    hostile, missing = hostile_files(fixtures)
    if not hostile:
        print(f"no hostile fixtures under {fixtures}", file=sys.stderr)
        return 2
    probe = args.probe or exitlib.build_probe()

    failures = [{"deck": f"hostile/{name}", "why": "listed in the manifest, missing from disk"} for name in missing]
    outcomes = collections.Counter()
    codes = collections.Counter()
    slowest = (0.0, "")
    biggest = (0, "")
    decks = [("hostile", p) for p in hostile] + [("other", p) for p in other_files(fixtures, args.local)]
    for kind, path in decks:
        name = f"{path.parent.name}/{path.name}"
        result = exitlib.bounded([str(probe), "survive", str(path)], args.seconds)
        slowest = max(slowest, (result["seconds"], name))
        biggest = max(biggest, (result["peak_bytes"], name))
        said = exitlib.answer(result)
        if said is None:
            failures.append({"deck": name, "why": exitlib.describe_crash(result)})
            continue
        if result["peak_bytes"] > args.memory_mb << 20:
            failures.append({"deck": name, "why": f"used {result['peak_bytes'] >> 20} MiB, over {args.memory_mb} MiB"})
        outcomes[(kind, said["outcome"])] += 1
        if said["outcome"] == "refused":
            codes[said.get("code", "?")] += 1

    note, loaded = exitlib.load_note()
    summary = {
        "decks": len(decks),
        "hostile": len(hostile),
        "failures": failures,
        "hostile_opened": outcomes[("hostile", "open")],
        "hostile_refused": outcomes[("hostile", "refused")],
        "others_opened": outcomes[("other", "open")],
        "others_refused": outcomes[("other", "refused")],
        "refusal_codes": dict(sorted(codes.items())),
        "slowest_seconds": slowest[0],
        "slowest": slowest[1],
        "peak_mib": biggest[0] >> 20,
        "peak_deck": biggest[1],
        "seconds_bound": args.seconds,
        "memory_bound_mb": args.memory_mb,
        "load": note,
        "loaded": loaded,
    }
    if args.json:
        pathlib.Path(args.json).write_text(json.dumps(summary, indent=1), encoding="utf8")
    for failure in failures:
        print(f"FAIL {failure['deck']}: {failure['why']}")
    print(f"{len(decks)} decks, {len(hostile)} hostile: {len(failures)} crash, hang or over a bound. "
          f"Hostile: {summary['hostile_opened']} opened, {summary['hostile_refused']} refused. "
          f"Slowest {slowest[0]} s ({slowest[1]}); peak {biggest[0] >> 20} MiB ({biggest[1]}). {note}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
