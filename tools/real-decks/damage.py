#!/usr/bin/env python3
"""`P1X-01`, docs/29's gate: damage real decks, open each, edit each, validate the result.

    python3 -I tools/real-decks/damage.py [--json FILE] [--seconds N] [--probe PATH]
                                          [--bases DECK ...] [--fixtures DIR]

Damage is generated into a temporary directory from committed decks, from a fixed seed, so
two runs are the same corpus: truncations, flipped bytes, bytes before and after the ZIP, a
lost central directory, removed parts, and broken XML at several places. `fixtures/damage`,
if it exists, joins in.

A deck passes if it opens and takes an edit that F4 and Microsoft's validator accept, or is
refused for a reason docs/29 allows: no ZIP and nothing recoverable, a compound file, or
over the whole-file limit. Anything else is reported, by damage kind and code. A deck that
opens with no text left to edit is counted apart: it opened, and its edit was not tested.

Exit: 0 if all pass, 1 if not, 2 if the run could not start.
"""

import argparse
import collections
import json
import pathlib
import random
import re
import subprocess
import sys
import tempfile
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import exitlib  # noqa: E402

SEED = 20261107
SECONDS = 60
# docs/29, "What still refuses".
ALLOWED_REFUSALS = {"CS-PKG-0005", "CS-OPC-0005", "CS-PKG-0001"}
BASES = ("generated/three-slides.pptx", "real/z21464709-souzou-ja.pptx", "real/sdk-animation.pptx")


def rewrite(source, change):
    """The package again, with `change(name, data)` giving new bytes, or None to drop a part."""
    target = tempfile.SpooledTemporaryFile()
    with zipfile.ZipFile(source) as old, zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as new:
        for info in old.infolist():
            data = change(info.filename, old.read(info))
            if data is not None:
                new.writestr(info.filename, data)
    target.seek(0)
    return target.read()


def first_slide(names):
    return min((n for n in names if re.fullmatch(r"ppt/slides/slide\d+\.xml", n)),
               key=lambda n: int(re.findall(r"\d+", n)[-1]))


def damaged(path, rng):
    """Yield `(kind, bytes)` for one deck."""
    source = path.read_bytes()
    size = len(source)
    for percent in (5, 25, 50, 75, 95, 99):
        yield f"container-truncate-{percent}", source[:size * percent // 100]
    for i in range(12):
        flipped = bytearray(source)
        at = rng.randrange(size)
        flipped[at] ^= 1 << rng.randrange(8)
        yield f"container-flip-{i}", bytes(flipped)
    yield "container-prepend", b"\x7fNOT A ZIP HEADER\n" * 8 + source
    yield "container-trailing", source + b"TRAILING BYTES" * 16
    at = source.rfind(b"PK\x01\x02", 0, size)
    first_central = source.find(b"PK\x01\x02")
    yield "container-no-central-directory", source[:first_central] if first_central > 0 else source[:at]
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
    slide = first_slide(names)
    rels = f"ppt/slides/_rels/{slide.rsplit('/', 1)[1]}.rels"
    for kind, part in (
        ("package-no-content-types", "[Content_Types].xml"),
        ("package-no-root-rels", "_rels/.rels"),
        ("package-no-presentation-rels", "ppt/_rels/presentation.xml.rels"),
        ("package-no-presentation", "ppt/presentation.xml"),
        ("package-no-slide", slide),
        ("package-no-slide-rels", rels),
        ("package-no-theme", "ppt/theme/theme1.xml"),
        ("package-no-master", "ppt/slideMasters/slideMaster1.xml"),
        ("package-no-layout", "ppt/slideLayouts/slideLayout1.xml"),
    ):
        yield kind, rewrite(path, lambda n, d, part=part: None if n == part else d)

    def in_part(part, edit):
        return rewrite(path, lambda n, d: edit(d) if n == part else d)

    targets = (("slide", slide), ("presentation", "ppt/presentation.xml"),
               ("content-types", "[Content_Types].xml"), ("root-rels", "_rels/.rels"))
    for label, part in targets:
        for percent in (25, 50, 75):
            yield (f"xml-{label}-truncate-{percent}",
                   in_part(part, lambda d, percent=percent: d[:len(d) * percent // 100]))
        yield f"xml-{label}-doctype", in_part(part, lambda d: re.sub(
            rb"(<\?xml[^>]*\?>\s*)", rb'\1<!DOCTYPE x [<!ENTITY a "aaaa"><!ENTITY b "&a;&a;">]>', d, count=1))
        yield f"xml-{label}-bad-char", in_part(part, lambda d: d[:len(d) // 2] + b"\x01&nope;<" + d[len(d) // 2:])
        yield f"xml-{label}-two-roots", in_part(part, lambda d: d + b"<second/>")
    yield "xml-slide-repeated-attribute", in_part(slide, lambda d: d.replace(b'<p:cNvPr id="', b'<p:cNvPr name="x" name="y" id="', 1))
    yield "xml-slide-undeclared-prefix", in_part(slide, lambda d: d.replace(b"<p:cSld>", b"<p:cSld><nope:thing/>", 1))
    yield "xml-slide-mismatched-tag", in_part(slide, lambda d: d.replace(b"</p:cSld>", b"</p:wrong>", 1))
    yield "model-bad-values", in_part(slide, lambda d: re.sub(rb'x="\d+"', b'x="banana"', d))
    yield "model-duplicate-ids", in_part(slide, lambda d: re.sub(rb'<p:cNvPr id="\d+"', b'<p:cNvPr id="2"', d))


def validate(paths):
    """Names failing F4 or the SDK validator. None for a validator that could not run."""
    bad = {}
    f4 = subprocess.run([sys.executable, "-I", str(exitlib.ROOT / "tools/f4/validate.py"), "--json", *map(str, paths)],
                        capture_output=True, text=True, check=False)
    if f4.returncode not in (0, 1):
        return None
    for deck in json.loads(f4.stdout):
        if deck["status"] != "pass":
            bad.setdefault(pathlib.Path(deck["deck"]).name, []).append("F4")
    sdk = subprocess.run([sys.executable, "-I", str(exitlib.ROOT / "tools/ooxml-sdk/validate.py"), "--json", *map(str, paths)],
                         capture_output=True, text=True, check=False)
    if not sdk.stdout.startswith("{"):
        return None
    for deck in json.loads(sdk.stdout)["decks"]:
        if deck.get("verdict") not in ("PASS", "KNOWN"):
            bad.setdefault(pathlib.Path(deck["deck"]).name, []).append("SDK")
    return bad


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--json")
    parser.add_argument("--seconds", type=float, default=SECONDS)
    parser.add_argument("--probe")
    parser.add_argument("--bases", nargs="*")
    parser.add_argument("--fixtures", default=str(exitlib.ROOT / "fixtures"))
    parser.add_argument("--no-validate", action="store_true", help="for a test: skip F4 and the SDK validator")
    args = parser.parse_args(argv[1:])

    fixtures = pathlib.Path(args.fixtures)
    bases = [pathlib.Path(b) for b in args.bases] if args.bases else [fixtures / b for b in BASES]
    missing = [str(b) for b in bases if not b.is_file()]
    if missing:
        print(f"base decks missing: {missing}", file=sys.stderr)
        return 2
    probe = args.probe or exitlib.build_probe()

    rng = random.Random(SEED)
    verdicts = collections.Counter()
    failures, edited = [], []
    with tempfile.TemporaryDirectory(prefix="damage-") as tmp:
        tmp = pathlib.Path(tmp)
        (tmp / "in").mkdir()
        (tmp / "out").mkdir()
        cases = []
        for base in bases:
            for kind, data in damaged(base, rng):
                path = tmp / "in" / f"{base.stem}--{kind}.pptx"
                path.write_bytes(data)
                cases.append((kind, path))
        extra = fixtures / "damage"
        if extra.is_dir():
            cases += [(p.stem, p) for p in exitlib.decks_under(extra)]
        for kind, path in cases:
            out = tmp / "out" / path.name
            result = exitlib.bounded([str(probe), "edit", str(path), str(out)], args.seconds)
            said = exitlib.answer(result)
            if said is None:
                verdicts["crash or hang"] += 1
                failures.append({"deck": path.name, "why": exitlib.describe_crash(result)})
            elif said["outcome"] == "refused":
                if said.get("code") in ALLOWED_REFUSALS:
                    verdicts["refused as docs/29 allows"] += 1
                else:
                    verdicts["refused against docs/29"] += 1
                    failures.append({"deck": path.name, "why": f"refused at {said.get('stage')}: {said.get('code')}"})
            elif said["outcome"] == "no-text":
                verdicts["opened, nothing to edit"] += 1
            else:
                verdicts["opened and edited"] += 1
                edited.append(out)
        invalid = {}
        if edited and not args.no_validate:
            invalid = validate(edited)
            if invalid is None:
                print("a validator could not run, so no edit was shown valid", file=sys.stderr)
                return 2
            for name, tools in sorted(invalid.items()):
                verdicts["edited, but not valid"] += 1
                failures.append({"deck": name, "why": "the edit does not validate: " + " ".join(tools)})

    by_kind = collections.Counter(re.sub(r"-\d+$", "", f["deck"].split("--", 1)[-1].removesuffix(".pptx")) for f in failures)
    codes = collections.Counter(f["why"].rsplit(": ", 1)[-1] for f in failures if f["why"].startswith("refused"))
    summary = {"cases": len(cases), "verdicts": dict(verdicts), "failed": len(failures), "by_kind": dict(by_kind),
               "codes": dict(codes), "failures": failures, "validated": not args.no_validate}
    if args.json:
        pathlib.Path(args.json).write_text(json.dumps(summary, indent=1), encoding="utf8")
    print(f"{len(cases)} damaged decks from {len(bases)} bases: {dict(verdicts)}")
    if failures:
        print(f"{len(failures)} fail. Codes {dict(codes)}. By kind: {dict(by_kind.most_common(12))}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
