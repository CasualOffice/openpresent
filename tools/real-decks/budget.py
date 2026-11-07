#!/usr/bin/env python3
"""`P1X-01`: a generated 500-slide deck against a stated time and memory budget.

    python3 -I tools/real-decks/budget.py [--json FILE] [--slides N] [--repeat N]
                                          [--probe PATH] [--validate] [--scale X]

The deck is written to a temporary directory, never committed: same input, same bytes. Each
slide has a title, five bullets, twelve text boxes, and on some slides a table or the one
shared picture, which is a text-heavy deck of an ordinary kind. It has no notes, charts or
media beyond that, so it measures the slide path and not those tiers.

Each stage runs in its own process from a release build, `--repeat` times. Time is judged on
the fastest run, since load only ever adds to it, and memory on the largest. Budgets are in
BUDGETS, each with the measurement it came from. Exit: 0 within budget, 1 over, 2 could not run.

`--scale` multiplies every budget, for a test to plant a failure.
"""

import argparse
import hashlib
import json
import pathlib
import struct
import subprocess
import sys
import tempfile
import zipfile
import zlib

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import exitlib  # noqa: E402

BASE = exitlib.ROOT / "fixtures/generated/three-slides.pptx"
FIXED_TIME = (2020, 1, 1, 0, 0, 0)
HANG_SECONDS = 600

# Stage: (milliseconds, MiB, what it stands for).
BUDGETS = {
    "open": (2000, 256, "open, import, index, read the first slide"),
    "open-all": (20000, 1024, "open, import, read every slide-like part"),
    "save": (5000, 256, "unedited save, returning the input bytes"),
    "edit": (5000, 256, "open, read one slide, change a run, save"),
}

NS = ('xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" '
      'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"')
DECL = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
SLIDE_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.slide+xml"


def png():
    """A small real PNG, so the deck carries media without bulk."""
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))
    size = 64
    raw = b"".join(b"\0" + bytes((x * 3 % 256, y * 3 % 256, 128) [i] for x in range(size) for i in range(3)) for y in range(size))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def box(shape_id, name, x, y, text, placeholder=""):
    paragraphs = "".join(f'<a:p><a:r><a:rPr lang="en-US" dirty="0"/><a:t>{t}</a:t></a:r></a:p>' for t in text)
    return (f'<p:sp><p:nvSpPr><p:cNvPr id="{shape_id}" name="{name}"/><p:cNvSpPr/><p:nvPr>{placeholder}</p:nvPr></p:nvSpPr>'
            f'<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="2743200" cy="914400"/></a:xfrm>'
            '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
            f'<p:txBody><a:bodyPr/><a:lstStyle/>{paragraphs}</p:txBody></p:sp>')


def table(shape_id):
    cell = '<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>cell</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>'
    row = f'<a:tr h="370840">{cell * 4}</a:tr>'
    grid = '<a:gridCol w="2032000"/>' * 4
    return (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{shape_id}" name="Table"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
            '<p:xfrm><a:off x="838200" y="4800000"/><a:ext cx="8128000" cy="1483360"/></p:xfrm>'
            '<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">'
            f'<a:tbl><a:tblPr/><a:tblGrid>{grid}</a:tblGrid>{row * 4}</a:tbl>'
            '</a:graphicData></a:graphic></p:graphicFrame>')


def picture(shape_id):
    return (f'<p:pic><p:nvPicPr><p:cNvPr id="{shape_id}" name="Picture"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
            '<p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>'
            '<p:spPr><a:xfrm><a:off x="9000000" y="300000"/><a:ext cx="914400" cy="914400"/></a:xfrm>'
            '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')


def slide_xml(number):
    shapes = [box(2, "Title 1", 838200, 365125, [f"Slide number {number}"], '<p:ph type="title"/>')]
    shapes.append(box(3, "Body", 838200, 1400000, [f"Point {i} of slide {number}, in ordinary words" for i in range(5)]))
    for i in range(12):
        shapes.append(box(4 + i, f"Box {i}", 400000 + (i % 4) * 2800000, 2500000 + (i // 4) * 800000, [f"Label {i}"]))
    if number % 10 == 0:
        shapes.append(table(20))
    if number % 5 == 0:
        shapes.append(picture(21))
    return (f'{DECL}<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>'
            f'<p:grpSpPr/>{"".join(shapes)}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>')


def slide_rels(number):
    picture_rel = f'<Relationship Id="rId2" Type="{REL}/image" Target="../media/image1.png"/>' if number % 5 == 0 else ""
    return (f'{DECL}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            f'<Relationship Id="rId1" Type="{REL}/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>{picture_rel}</Relationships>')


def make_deck(path, slides):
    """Write the deck. Master, layout and theme come from a fixture that already validates."""
    with zipfile.ZipFile(BASE) as base:
        kept = {n: base.read(n) for n in base.namelist() if not n.startswith("ppt/slides/")}
    types = kept["[Content_Types].xml"].decode("utf8")
    overrides = "".join(f'<Override PartName="/ppt/slides/slide{n}.xml" ContentType="{SLIDE_TYPE}"/>'
                        for n in range(1, slides + 1))
    head, tail = types.split("</Types>")
    head = "".join(t + ">" for t in head.split(">") if t and "/ppt/slides/" not in t)
    if 'Extension="png"' not in head:
        head = head.replace('<Default ', '<Default Extension="png" ContentType="image/png"/><Default ', 1)
    kept["[Content_Types].xml"] = (head + overrides + "</Types>" + tail).encode("utf8")

    presentation = kept["ppt/presentation.xml"].decode("utf8")
    start, rest = presentation.split("<p:sldIdLst>")
    _, end = rest.split("</p:sldIdLst>")
    ids = "".join(f'<p:sldId id="{255 + n}" r:id="rId{100 + n}"/>' for n in range(1, slides + 1))
    kept["ppt/presentation.xml"] = f"{start}<p:sldIdLst>{ids}</p:sldIdLst>{end}".encode("utf8")

    rels = kept["ppt/_rels/presentation.xml.rels"].decode("utf8")
    head = "".join(t + ">" for t in rels.split("</Relationships>")[0].split(">") if t and "slides/slide" not in t)
    links = "".join(f'<Relationship Id="rId{100 + n}" Type="{REL}/slide" Target="slides/slide{n}.xml"/>'
                    for n in range(1, slides + 1))
    kept["ppt/_rels/presentation.xml.rels"] = (head + links + "</Relationships>").encode("utf8")

    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as out:
        def put(name, data):
            info = zipfile.ZipInfo(name, FIXED_TIME)
            info.compress_type = zipfile.ZIP_DEFLATED
            out.writestr(info, data)
        for name in sorted(kept):
            put(name, kept[name])
        put("ppt/media/image1.png", png())
        for n in range(1, slides + 1):
            put(f"ppt/slides/slide{n}.xml", slide_xml(n).encode("utf8"))
            put(f"ppt/slides/_rels/slide{n}.xml.rels", slide_rels(n).encode("utf8"))


def validate(paths):
    """F4 and the SDK validator, so the deck is a real one and the edit a valid file."""
    problems = []
    f4 = subprocess.run([sys.executable, "-I", str(exitlib.ROOT / "tools/f4/validate.py"), "--json", *map(str, paths)],
                        capture_output=True, text=True, check=False)
    sdk = subprocess.run([sys.executable, "-I", str(exitlib.ROOT / "tools/ooxml-sdk/validate.py"), "--json", *map(str, paths)],
                         capture_output=True, text=True, check=False)
    if f4.returncode != 0:
        problems.append(f"F4 exit {f4.returncode}")
    if sdk.returncode != 0 or not sdk.stdout.startswith("{"):
        problems.append(f"SDK exit {sdk.returncode}: {sdk.stdout[:120]}")
    return problems


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--json")
    parser.add_argument("--slides", type=int, default=500)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--probe")
    parser.add_argument("--validate", action="store_true")
    parser.add_argument("--scale", type=float, default=1.0)
    args = parser.parse_args(argv[1:])
    probe = args.probe or exitlib.build_probe()

    failures, stages = [], {}
    with tempfile.TemporaryDirectory(prefix="budget-") as tmp:
        deck = pathlib.Path(tmp) / "budget-500.pptx"
        make_deck(deck, args.slides)
        size = deck.stat().st_size
        sha = hashlib.sha256(deck.read_bytes()).hexdigest()
        edited = pathlib.Path(tmp) / "edited.pptx"
        for stage, (ms, mib, meaning) in BUDGETS.items():
            command = [str(probe), "budget", stage, str(deck)] + ([str(edited)] if stage == "edit" else [])
            times, peak = [], 0
            for _ in range(args.repeat):
                result = exitlib.bounded(command, HANG_SECONDS)
                said = exitlib.answer(result)
                if said is None:
                    failures.append(f"{stage}: {exitlib.describe_crash(result)}")
                    break
                if said["outcome"] != "ok":
                    failures.append(f"{stage}: refused at {said.get('stage')}: {said.get('code')}")
                    break
                if stage == "save" and said.get("unedited_same") is not True:
                    failures.append("save: an unedited save did not return the input bytes")
                if stage in ("open", "open-all") and said.get("slides") != args.slides:
                    failures.append(f"{stage}: read {said.get('slides')} slides of {args.slides}")
                if stage == "edit" and said.get("changed_parts") != 1:
                    failures.append(f"edit: changed {said.get('changed_parts')} parts, not 1")
                times.append(said["ms"])
                peak = max(peak, result["peak_bytes"])
            if not times:
                continue
            best, peak_mib = min(times), peak / (1 << 20)
            limit_ms, limit_mib = ms * args.scale, mib * args.scale
            over = best > limit_ms or peak_mib > limit_mib
            stages[stage] = {"ms": times, "best_ms": best, "peak_mib": round(peak_mib, 1), "budget_ms": limit_ms,
                             "budget_mib": limit_mib, "within": not over, "meaning": meaning}
            print(f"{'ok  ' if not over else 'OVER'} {stage}: {best} ms of {limit_ms:g}, "
                  f"{peak_mib:.0f} MiB of {limit_mib:g} ({meaning})")
            if over:
                failures.append(f"{stage}: {best} ms of {limit_ms:g}, {peak_mib:.0f} MiB of {limit_mib:g}")
        if args.validate and not failures:
            failures += validate([deck, edited])

    note, loaded = exitlib.load_note()
    summary = {"slides": args.slides, "bytes": size, "sha256": sha, "stages": stages, "failures": failures,
               "load": note, "loaded": loaded, "validated": args.validate}
    if args.json:
        pathlib.Path(args.json).write_text(json.dumps(summary, indent=1), encoding="utf8")
    print(f"{args.slides} slides, {size} bytes, sha256 {sha[:12]}. {note}")
    for failure in failures:
        print(f"FAIL {failure}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
