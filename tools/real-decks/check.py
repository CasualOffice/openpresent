#!/usr/bin/env python3
"""F-007: import, forced rewrite, F4 and the SDK validator over real decks.

    python3 -I tools/real-decks/check.py --report out.json [--keep-exports DIR] DIR_OR_DECK ...

`--keep-exports` keeps each forced rewrite, for a LibreOffice reopen in a container.

Reports every failure per deck and stops at none of them. Findings are the point: these
are decks we did not write. Exits 0 if every deck passed every stage, 1 if not, 2 if a
tool could not run. Decks are untrusted, so run this with `-I`, from outside the
directory holding them; nothing here executes or extracts anything from a deck.
Builds the runner under target/real-decks.
"""

import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RUNNER = ROOT / "target/real-decks/release/casual-slide-real-decks"
SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}


def decks_in(arguments):
    found = []
    for argument in arguments:
        path = pathlib.Path(argument)
        found += sorted(p for p in path.iterdir() if p.suffix.lower() in SUFFIXES) if path.is_dir() else [path]
    return found


def f4(paths):
    """F4's result per deck name. Exit 1 means a deck failed, which is an answer."""
    done = subprocess.run(
        [sys.executable, "-I", str(ROOT / "tools/f4/validate.py"), "--json", *map(str, paths)],
        capture_output=True, text=True, check=False,
    )
    if done.returncode not in (0, 1):
        raise SystemExit(f"F4 could not run: {done.stderr[-500:]}")
    return {d["deck"]: d for d in json.loads(done.stdout)}


def summary(result):
    """What F4 said, without the line numbers, so two runs can be compared."""
    if result is None:
        return None
    return {
        "status": result["status"],
        "refused": result.get("refused"),
        "parts": {p["part"]: (p["status"], p.get("errors")) for p in result["parts"]},
    }


def sdk_of(paths):
    """The SDK validator's result per deck name; empty if it could not run."""
    done = subprocess.run(
        [sys.executable, "-I", str(ROOT / "tools/ooxml-sdk/validate.py"), "--json", *map(str, paths)],
        capture_output=True, text=True, check=False,
    )
    if done.stdout.startswith("{"):
        return {d["deck"]: d for d in json.loads(done.stdout)["decks"]}
    print(done.stdout.strip()[:200])
    return {}


def main(argv):
    args = argv[1:]
    report = None
    if "--report" in args:
        at = args.index("--report")
        report = args[at + 1]
        del args[at:at + 2]
    keep = None
    if "--keep-exports" in args:
        at = args.index("--keep-exports")
        keep = pathlib.Path(args[at + 1])
        del args[at:at + 2]
    decks = decks_in(args)
    if not decks:
        print(__doc__, file=sys.stderr)
        return 2

    built = subprocess.run(
        ["cargo", "build", "--release", "--locked", "--manifest-path", str(HERE / "Cargo.toml"),
         "--target-dir", str(ROOT / "target/real-decks")],
        capture_output=True, text=True, check=False,
    )
    if built.returncode != 0:
        print(built.stderr[-2000:], file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory(prefix="real-decks-") as tmp:
        out = pathlib.Path(tmp)
        ran = subprocess.run([str(RUNNER), str(out), *map(str, decks)], capture_output=True, text=True, check=False)
        engine = {}
        for line in ran.stdout.splitlines():
            record = json.loads(line)
            engine[record.pop("deck")] = record
        missing = [d.name for d in decks if d.name not in engine]
        original = f4(decks)
        rewritten_paths = sorted(out.iterdir())
        rewritten = f4(rewritten_paths) if rewritten_paths else {}
        sdk = sdk_of(decks)
        sdk_after = sdk_of(rewritten_paths) if rewritten_paths else {}
        if keep:
            keep.mkdir(parents=True, exist_ok=True)
            for path in rewritten_paths:
                shutil.copyfile(path, keep / path.name)

    features = json.loads(subprocess.run(
        [sys.executable, "-I", str(HERE / "features.py"), "--json", *map(str, decks)],
        capture_output=True, text=True, check=True,
    ).stdout)
    features = {f["deck"]: f for f in features}

    results = []
    for deck in decks:
        name = deck.name
        stages = engine.get(name, {})
        before, after = original.get(name), rewritten.get(name)
        results.append({
            "deck": name,
            "engine": stages,
            "engine_ok": name not in missing and all(s["status"] == "ok" for s in stages.values())
                         and {"open", "import", "unedited_save", "forced_rewrite"} <= set(stages),
            "f4": before and before["status"],
            "f4_refused": before and before.get("refused"),
            "f4_failures": [
                {"part": p["part"], "status": p["status"], "errors": p.get("errors", [])}
                for p in (before or {}).get("parts", []) if p["status"] not in ("valid", "no-schema", "recorded")
            ],
            "f4_no_schema": sum(p["status"] == "no-schema" for p in (before or {}).get("parts", [])),
            "f4_same_after_rewrite": None if after is None else summary(before) == summary(after),
            "f4_after": after and after["status"],
            "sdk_after": sdk_after.get(name, {}).get("verdict"),
            "sdk": sdk.get(name, {}).get("status"),
            "sdk_reason": sdk.get(name, {}).get("reason"),
            "sdk_total": sdk.get(name, {}).get("total"),
            "sdk_errors": sdk.get(name, {}).get("errors", []),
            "features": features.get(name, {}),
        })
    if report:
        pathlib.Path(report).write_text(json.dumps(results, indent=1, ensure_ascii=False), encoding="utf8")

    failed = 0
    for r in results:
        bad = [f"{k}={v['status']}" for k, v in r["engine"].items() if v["status"] != "ok"]
        clean = r["engine_ok"] and r["f4"] == "pass" and r["sdk"] == "pass" and r["f4_same_after_rewrite"]
        failed += not clean
        print(f"{'ok  ' if clean else 'FAIL'} {r['deck']}: engine {'ok' if r['engine_ok'] else ' '.join(bad) or 'missing'}; "
              f"F4 {r['f4']} ({len(r['f4_failures'])} parts); same after rewrite {r['f4_same_after_rewrite']}; "
              f"SDK {r['sdk']} ({r['sdk_total']})")
    print(f"{len(results)} decks, {failed} with a failure")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
