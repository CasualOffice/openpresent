#!/usr/bin/env python3
"""F-006: Microsoft's Open XML SDK validator over decks.

The closest stand-in for PowerPoint's strictness we can run without PowerPoint
(`DEC-04`). The SDK's schemas are Office's reading of the formats, where F4's are
Ecma's, so the two disagree in places, and each disagreement is information:
`false-positives.tsv` records every one we have judged, with the reason.

    python3 tools/ooxml-sdk/validate.py fixtures/generated fixtures/producers
    python3 tools/ooxml-sdk/validate.py --json deck.pptx
    python3 tools/ooxml-sdk/validate.py --self-test     # must reject a seeded invalid part
    python3 tools/ooxml-sdk/validate.py --no-known ...  # report recorded findings as failures

Needs `dotnet` (SDK 10). Without it, this prints one SKIPPED line and exits 0, unless
OOXML_SDK_REQUIRED is set, as CI sets it on Linux, where a missing tool fails.

The validator is DocumentFormat.OpenXml, pinned in Validator.csproj and
packages.lock.json, validating as `Microsoft365` unless `--version` says otherwise.

Decks are untrusted. Each is bounded here, from the central directory alone, before the
SDK sees it, with F4's bounds; `fixtures/hostile` is refused outright. Diagnostics name
parts, paths and rule ids; values quoted from the file are redacted.

Exit: 0 if every deck passed or failed only as recorded, 1 if not, 2 if it could not run.
"""

import fnmatch
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROJECT = HERE / "Validator.csproj"
DLL = ROOT / "target/ooxml-sdk/bin/Release/net10.0/Validator.dll"
KNOWN = HERE / "false-positives.tsv"
DECK_SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}
SDK_VERSION = "3.5.1"

# F4's bounds (tools/f4/validate.py), which are OoxmlLimits' or tighter.
MAX_INPUT = 1 << 30
MAX_ENTRIES = 100_000
MAX_PART_BYTES = 256 << 20
MAX_TOTAL = 4 << 30
MAX_RATIO = 1_000


def find_dotnet():
    for candidate in (
        shutil.which("dotnet"),
        os.environ.get("DOTNET_ROOT") and str(pathlib.Path(os.environ["DOTNET_ROOT"]) / "dotnet"),
        "/opt/homebrew/bin/dotnet",
        "/usr/local/share/dotnet/dotnet",
        str(pathlib.Path.home() / ".dotnet/dotnet"),
    ):
        if candidate and pathlib.Path(candidate).is_file():
            return candidate
    return None


def environment():
    env = dict(os.environ)
    env.update(DOTNET_CLI_TELEMETRY_OPTOUT="1", DOTNET_NOLOGO="1", DOTNET_SKIP_FIRST_TIME_EXPERIENCE="1")
    return env


def build(dotnet):
    sources = [PROJECT, HERE / "Program.cs", HERE / "packages.lock.json", HERE / "Directory.Build.props"]
    if DLL.is_file() and all(DLL.stat().st_mtime >= s.stat().st_mtime for s in sources):
        return
    done = subprocess.run(
        [dotnet, "build", str(PROJECT), "-c", "Release", "--nologo", "-v", "quiet"],
        capture_output=True, text=True, env=environment(), check=False,
    )
    if done.returncode != 0:
        print(done.stdout[-3000:], done.stderr[-2000:], file=sys.stderr)
        raise SystemExit(2)


class Refused(Exception):
    pass


def admit(path):
    """The archive's shape, from the central directory, before anything inflates."""
    data_len = path.stat().st_size
    if data_len > MAX_INPUT:
        raise Refused("over the input bound")
    with open(path, "rb") as f:
        if f.read(8) == bytes.fromhex("d0cf11e0a1b11ae1"):
            raise Refused("a Compound File")
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
    except (zipfile.BadZipFile, OSError, EOFError, RuntimeError, NotImplementedError) as err:
        raise Refused(f"not a readable package: {type(err).__name__}") from err
    if len(infos) > MAX_ENTRIES:
        raise Refused("too many entries")
    names = [i.filename for i in infos]
    if len(set(names)) != len(names):
        raise Refused("duplicate entries")
    total = 0
    for info in infos:
        total += info.file_size
        if info.file_size > MAX_PART_BYTES and info.filename.lower().endswith((".xml", ".rels")):
            raise Refused("an XML part over the size bound")
        if info.compress_size and info.file_size // info.compress_size > MAX_RATIO:
            raise Refused("a part over the expansion bound")
    if total > MAX_TOTAL:
        raise Refused("over the total size bound")


def expand(arguments):
    for argument in arguments:
        path = pathlib.Path(argument)
        resolved = path.resolve().parts
        if "hostile" in resolved and "fixtures" in resolved:
            raise SystemExit("ooxml-sdk: refusing fixtures/hostile: no external tool runs on it")
        if path.is_dir():
            yield from sorted(p for p in path.iterdir() if p.suffix.lower() in DECK_SUFFIXES)
        else:
            yield path


def known_findings():
    rows = []
    if KNOWN.is_file():
        for line in KNOWN.read_text(encoding="utf8").splitlines():
            if not line.strip() or line.startswith("#"):
                continue
            cols = line.split("\t")
            if len(cols) != 6:
                raise SystemExit(f"{KNOWN.name}: a row needs 6 tab-separated columns: {line[:60]}")
            rows.append(dict(zip(("deck", "id", "part", "path", "kind", "reason"), cols)))
    return rows


def matches(row, deck, error):
    return (
        fnmatch.fnmatch(deck, row["deck"])
        and row["id"] == error["id"]
        and fnmatch.fnmatch(error["part"], row["part"])
        and (row["path"] == "*" or re.search(row["path"], error["path"]) is not None)
    )


def run(dotnet, decks, version):
    results = []
    admitted = []
    for deck in decks:
        try:
            admit(deck)
            admitted.append(deck)
        except Refused as err:
            results.append({"deck": deck.name, "status": "refused", "reason": f"bounds: {err}", "errors": [], "total": 0})
    if admitted:
        done = subprocess.run(
            [dotnet, str(DLL), version, *map(str, admitted)],
            capture_output=True, text=True, env=environment(), check=False,
        )
        if done.returncode not in (0, 1):
            print(done.stderr[-2000:], file=sys.stderr)
            raise SystemExit(2)
        results += [json.loads(line) for line in done.stdout.splitlines() if line.strip()]
    return results


def judge(results, use_known):
    rows = known_findings() if use_known else []
    used = set()
    for result in results:
        unexplained = []
        explained = []
        entries = list(result["errors"])
        if result["status"] == "refused":
            entries = [{"id": f"refused:{result['reason']}", "part": "*", "path": "", "type": "", "description": ""}]
        for error in entries:
            hit = next((i for i, row in enumerate(rows) if matches(row, result["deck"], error)), None)
            if hit is None:
                unexplained.append(error)
            else:
                used.add(hit)
                explained.append((error, rows[hit]))
        if result.get("total", 0) > len(result["errors"]):
            unexplained.append({"id": "truncated", "part": "", "path": "",
                                "description": f"{result['total']} errors, only {len(result['errors'])} kept"})
        result["unexplained"] = unexplained
        result["explained"] = explained
        result["verdict"] = "FAIL" if unexplained else ("KNOWN" if explained else "PASS")
    return [rows[i] for i in range(len(rows)) if i not in used]


def print_text(results, unused):
    for r in results:
        line = f"{r['deck']}: {r['verdict']}"
        if r["status"] == "refused":
            line += f" (refused: {r['reason']})"
        print(line)
        for e in r["unexplained"]:
            print(f"  {e['id']}  {e['part']}  {e['path']}\n      {e['description']}")
        for e, row in r["explained"]:
            print(f"  known {row['kind']}: {e['id']}  {e['part']}  {e['path']}")
    failed = sum(r["verdict"] == "FAIL" for r in results)
    known = sum(r["verdict"] == "KNOWN" for r in results)
    print(f"{len(results)} decks, {failed} failed, {known} only as recorded in {KNOWN.name}")
    for row in unused:
        print(f"note: {KNOWN.name} row not seen in this run: {row['deck']} {row['id']} {row['part']}")


def self_test(dotnet, version):
    """A seeded invalid part must fail, on that part, with nothing from the file quoted."""
    source = ROOT / "fixtures/generated/minimal.pptx"
    marker = "SEEDEDMARKER7f3a"
    with tempfile.TemporaryDirectory() as tmp:
        bad = pathlib.Path(tmp) / "seeded.pptx"
        with zipfile.ZipFile(source) as src, zipfile.ZipFile(bad, "w", zipfile.ZIP_DEFLATED) as dst:
            for info in src.infolist():
                data = src.read(info)
                if info.filename == "ppt/slides/slide1.xml":
                    text = data.decode("utf8")
                    seeded = text.replace("<p:cSld>", f'<p:cSld name="x"><p:bogus/>', 1)
                    seeded = seeded.replace("<p:sld ", f'<p:sld show="{marker}" ', 1)
                    assert seeded != text, "the seed did not apply"
                    data = seeded.encode("utf8")
                dst.writestr(info, data)
        clean, seeded = run(dotnet, [source, bad], version)
    problems = []
    if clean["status"] != "pass":
        problems.append(f"the unseeded deck did not pass: {clean}")
    if seeded["status"] != "fail":
        problems.append(f"the seeded deck was not rejected: {seeded['status']}")
    parts = {e["part"] for e in seeded["errors"]}
    if parts != {"/ppt/slides/slide1.xml"}:
        problems.append(f"errors were not confined to the seeded part: {sorted(parts)}")
    ids = {e["id"] for e in seeded["errors"]}
    if not ids & {"Sch_InvalidElementContentExpectingComplex", "Sch_UnexpectedElementContentExpectingComplex"}:
        problems.append(f"the bogus element was not reported: {sorted(ids)}")
    if marker in json.dumps(seeded):
        problems.append("a value from the file reached the report")
    if problems:
        print("ooxml-sdk self-test failed:", *problems, sep="\n  ", file=sys.stderr)
        return 1
    print(f"ooxml-sdk self-test: a seeded invalid part is rejected ({', '.join(sorted(ids))}), "
          f"on that part alone, and the seeded value is not quoted")
    return 0


def main(argv):
    args = list(argv[1:])
    as_json = "--json" in args
    use_known = "--no-known" not in args
    version = "Microsoft365"
    if "--version" in args:
        version = args[args.index("--version") + 1]
        del args[args.index("--version"):args.index("--version") + 2]
    args = [a for a in args if a not in ("--json", "--no-known")]

    dotnet = find_dotnet()
    if dotnet is None:
        if os.environ.get("OOXML_SDK_REQUIRED"):
            print("ooxml-sdk: OOXML_SDK_REQUIRED is set and dotnet is not installed", file=sys.stderr)
            return 2
        print("SKIPPED: dotnet is not installed, so Microsoft's Open XML SDK validator did not run (F-006)")
        return 0
    build(dotnet)

    if args == ["--self-test"]:
        return self_test(dotnet, version)
    if not args:
        print(__doc__, file=sys.stderr)
        return 2
    results = run(dotnet, list(expand(args)), version)
    unused = judge(results, use_known)
    if as_json:
        for r in results:
            r["explained"] = [{"error": e, "kind": row["kind"], "reason": row["reason"]} for e, row in r["explained"]]
        print(json.dumps({"sdk": SDK_VERSION, "version": version, "decks": results}, indent=1))
    else:
        print(f"Open XML SDK {SDK_VERSION}, validating as {version}")
        print_text(results, unused)
    return 1 if any(r["verdict"] == "FAIL" for r in results) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
