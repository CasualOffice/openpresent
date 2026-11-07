#!/usr/bin/env python3
"""`CORP-01`: count the corpus by producer, version, language, script and size, against targets.

    python3 -I tools/real-decks/census.py [--local DIR] [--targets FILE] [--json FILE]

Counts fixtures/real and fixtures/producers, which are committed, and any local directory
filled by fetch.py. **A deck counts only if its provenance is recorded:** a row in
fixtures/real/README.md whose SHA-256 matches, or for fixtures/producers a row in the
manifest. Any other deck is named and not counted. Targets, with why each is what it is,
are in targets.tsv: `corpus` counts all decks here, `committed` only those in git.

Decks are untrusted: read only through features.py, in memory, under bounds. Prints names
and counts, never document text.

Exit: 0 if every target is met, 1 if one is not, 2 if the census could not run.
"""

import argparse
import collections
import csv
import hashlib
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import exitlib  # noqa: E402
import features  # noqa: E402

ROW = re.compile(r"^\| `([\w.-]+)` \| (?P<committed>[^|]*) \| \[source\]\(([^)\s]+)\) \|.*?\| `([0-9a-f]{64})` \|")


def provenance(root):
    """Name to `(sha256, url)` from the provenance table; path to sha256 from the manifest."""
    table = {}
    for line in (root / "fixtures/real/README.md").read_text(encoding="utf8").splitlines():
        match = ROW.match(line)
        if match:
            table[match.group(1)] = (match.group(4), match.group(3))
    manifest = {}
    for line in (root / "fixtures/manifest.tsv").read_text(encoding="utf8").splitlines()[1:]:
        cols = line.split("\t")
        manifest[cols[0]] = cols[1]
    return table, manifest


def primary_language(scan):
    languages = scan.get("languages") or []
    return languages[0].split("-")[0].lower() if languages else ""


def slide_bucket(n):
    return "1-5" if n <= 5 else "6-20" if n <= 20 else "21-50" if n <= 50 else "51+"


def byte_bucket(n):
    return "under 1 MB" if n < 1_000_000 else "1 to 10 MB" if n <= 10_000_000 else "over 10 MB"


def facts(scan, size):
    """Every `(axis, value)` a deck counts toward."""
    found = [("producer", scan.get("producer", "Unknown")), ("slides", slide_bucket(scan.get("slides", 0))),
             ("bytes", byte_bucket(size))]
    if scan.get("version"):
        found.append(("version", scan["version"]))
    if primary_language(scan):
        found.append(("language", primary_language(scan)))
    found += [("script", name) for name in scan.get("scripts", {})]
    feats = scan.get("features", {})
    found += [("feature", name) for name in ("morph", "zoom", "math", "smartart", "charts", "chartex", "tables", "ink",
                                             "ole", "video", "audio", "sections", "model3d", "svg", "modern_comments")
              if feats.get(name)]
    if scan.get("strict"):
        found.append(("feature", "strict"))
        if scan.get("strict_form") == "percent" and str(scan.get("producer", "")).startswith("PowerPoint"):
            found.append(("feature", "strict-saved"))
    return found


def census(root, local):
    table, manifest = provenance(root)
    uncounted, errors, decks = [], [], []
    candidates = [(p, True) for p in exitlib.decks_under(root / "fixtures/real")]
    candidates += [(p, True) for p in exitlib.decks_under(root / "fixtures/producers")]
    if local:
        candidates += [(p, False) for p in exitlib.decks_under(local)]
    seen = set()
    for path, committed in candidates:
        sha = hashlib.sha256(path.read_bytes()).hexdigest()
        if sha in seen:
            continue
        seen.add(sha)
        if path.parent.name == "producers":
            recorded = manifest.get(f"producers/{path.name}") == sha
        else:
            recorded = table.get(path.name, ("",))[0] == sha
        if not recorded:
            uncounted.append(path.name)
            continue
        try:
            scan = features.scan(str(path))
        except Exception as err:  # noqa: BLE001 - a hostile file may raise anything; the deck is named
            errors.append(f"{path.name}: {type(err).__name__}")
            continue
        decks.append({"name": path.name, "committed": committed, "bytes": path.stat().st_size, "scan": scan,
                      "facts": facts(scan, path.stat().st_size)})
    return decks, uncounted, errors


def count(decks):
    """`(scope, axis, value)` to count, and each axis's distinct values."""
    counts = collections.Counter()
    for deck in decks:
        scopes = ("corpus", "committed") if deck["committed"] else ("corpus",)
        for scope in scopes:
            counts[(scope, "total", "decks")] += 1
            if deck["committed"]:
                counts[(scope, "total", "committed")] += 1
            for axis, value in set(map(tuple, deck["facts"])):
                counts[(scope, axis, value)] += 1
    return counts


def judge(counts, targets):
    results = []
    for target in targets:
        wanted, scope, axis, value = int(target["min"]), target["scope"], target["axis"], target["value"]
        if value == "distinct":
            have = sum(1 for (s, a, _) in counts if s == scope and a == axis)
        else:
            have = counts[(scope, axis, value)]
        results.append(dict(target, have=have, met=have >= wanted))
    return results


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--local")
    parser.add_argument("--targets", default=str(HERE / "targets.tsv"))
    parser.add_argument("--json")
    parser.add_argument("--root", default=str(exitlib.ROOT))
    args = parser.parse_args(argv[1:])
    try:
        with open(args.targets, encoding="utf8", newline="") as handle:
            targets = list(csv.DictReader(handle, delimiter="\t"))
        decks, uncounted, errors = census(pathlib.Path(args.root), args.local)
    except (OSError, KeyError, ValueError) as err:
        print(f"census could not run: {err}", file=sys.stderr)
        return 2
    if not targets or not decks:
        print("census could not run: no targets or no decks", file=sys.stderr)
        return 2

    counts = count(decks)
    results = judge(counts, targets)
    for axis in ("producer", "version", "language", "script", "slides", "bytes", "feature"):
        row = sorted(((v, n) for (s, a, v), n in counts.items() if s == "corpus" and a == axis), key=lambda x: (-x[1], x[0]))
        print(f"{axis}: " + ", ".join(f"{v} {n}" for v, n in row))
    committed = sum(d["committed"] for d in decks)
    print(f"decks: {len(decks)} counted, {committed} committed, {len(decks) - committed} local")
    for name in uncounted:
        print(f"NOT COUNTED {name}: no matching provenance row")
    for error in errors:
        print(f"NOT COUNTED {error}: unreadable")
    under = [r for r in results if not r["met"]]
    for r in under:
        print(f"UNDER {r['axis']} {r['value']}: {r['have']} of {r['min']} ({r['scope']}). {r['why']}")
    print(f"{len(results) - len(under)} of {len(results)} targets met")
    if args.json:
        pathlib.Path(args.json).write_text(json.dumps({
            "decks": [{k: d[k] for k in ("name", "committed", "bytes")} | {"producer": d["scan"].get("producer")} for d in decks],
            "uncounted": uncounted, "errors": errors, "results": results,
        }, indent=1, ensure_ascii=False), encoding="utf8")
    return 1 if under else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
