#!/usr/bin/env python3
"""Compare this engine's outlines with what LibreOffice actually draws.

Not part of CI: it needs `soffice`. For each preset named, this writes a one-shape
.pptx with no stroke, converts it to SVG with soffice, flattens the shape's paths,
and reports the Hausdorff distance to the same preset evaluated here:

    cargo run -q -p casual-slide-geometry --example dump_geometry -- 5400000 3600000 cubics > dump.jsonl
    python3 tools/geometry-presets/render_crosscheck.py dump.jsonl WORKDIR [preset ...]

A small distance says LibreOffice and this engine draw about the same outline at
this size and at default adjusts. It says nothing about PowerPoint.
"""
import os
import json
import math
import pathlib
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
import zipfile

W, H = 5_400_000, 3_600_000
OFF_X, OFF_Y = 900_000, 900_000
EMU_PER_SVG_UNIT = 360  # soffice's SVG uses 1/100 mm
DEFAULT = (
    "rect ellipse roundRect rightArrow upArrow downArrow upDownArrow "
    "wedgeRoundRectCallout star5 chevron heart pie moon cloud circularArrow can "
    "flowChartDocument wave teardrop"
).split()

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/><Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/><Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/><Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/><Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/></Types>"""
RELS = 'xmlns="http://schemas.openxmlformats.org/package/2006/relationships"'
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
NS = (
    'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
    f'xmlns:r="{R}" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"'
)
TREE = '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>'
COLORS = "".join(
    f'<a:{n}><a:srgbClr val="000000"/></a:{n}>'
    for n in "dk1 lt1 dk2 lt2 accent1 accent2 accent3 accent4 accent5 accent6 hlink folHlink".split()
)
FONT = '<a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/>'
FILLS = '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>' * 3
LINES = '<a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>' * 3
EFFECTS = "<a:effectStyle><a:effectLst/></a:effectStyle>" * 3
THEME = (
    f'<?xml version="1.0" encoding="UTF-8"?><a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="t">'
    f'<a:themeElements><a:clrScheme name="c">{COLORS}</a:clrScheme><a:fontScheme name="f"><a:majorFont>{FONT}</a:majorFont>'
    f'<a:minorFont>{FONT}</a:minorFont></a:fontScheme><a:fmtScheme name="s"><a:fillStyleLst>{FILLS}</a:fillStyleLst>'
    f"<a:lnStyleLst>{LINES}</a:lnStyleLst><a:effectStyleLst>{EFFECTS}</a:effectStyleLst><a:bgFillStyleLst>{FILLS}</a:bgFillStyleLst>"
    "</a:fmtScheme></a:themeElements></a:theme>"
)


def write_deck(path, preset):
    shape = (
        '<p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>'
        f'<a:xfrm><a:off x="{OFF_X}" y="{OFF_Y}"/><a:ext cx="{W}" cy="{H}"/></a:xfrm>'
        f'<a:prstGeom prst="{preset}"><a:avLst/></a:prstGeom>'
        '<a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:ln><a:noFill/></a:ln></p:spPr></p:sp>'
    )
    parts = {
        "[Content_Types].xml": CONTENT_TYPES,
        "_rels/.rels": f'<Relationships {RELS}><Relationship Id="rId1" Type="{R}/officeDocument" Target="ppt/presentation.xml"/></Relationships>',
        "ppt/presentation.xml": f'<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:sldSz cx="9144000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>',
        "ppt/_rels/presentation.xml.rels": f'<Relationships {RELS}><Relationship Id="rId1" Type="{R}/slideMaster" Target="slideMasters/slideMaster1.xml"/><Relationship Id="rId2" Type="{R}/slide" Target="slides/slide1.xml"/></Relationships>',
        "ppt/slides/slide1.xml": f"<p:sld {NS}><p:cSld><p:spTree>{TREE}{shape}</p:spTree></p:cSld></p:sld>",
        "ppt/slides/_rels/slide1.xml.rels": f'<Relationships {RELS}><Relationship Id="rId1" Type="{R}/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>',
        "ppt/slideLayouts/slideLayout1.xml": f"<p:sldLayout {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld></p:sldLayout>",
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels": f'<Relationships {RELS}><Relationship Id="rId1" Type="{R}/slideMaster" Target="../slideMasters/slideMaster1.xml"/></Relationships>',
        "ppt/slideMasters/slideMaster1.xml": f'<p:sldMaster {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst></p:sldMaster>',
        "ppt/slideMasters/_rels/slideMaster1.xml.rels": f'<Relationships {RELS}><Relationship Id="rId1" Type="{R}/slideLayout" Target="../slideLayouts/slideLayout1.xml"/><Relationship Id="rId2" Type="{R}/theme" Target="../theme/theme1.xml"/></Relationships>',
        "ppt/theme/theme1.xml": THEME,
    }
    with zipfile.ZipFile(path, "w") as z:
        for name, text in parts.items():
            z.writestr(name, text)


def sample_cubic(a, b, c, d, out, steps=48):
    for i in range(1, steps + 1):
        t = i / steps
        u = 1 - t
        out.append(
            (
                u**3 * a[0] + 3 * u * u * t * b[0] + 3 * u * t * t * c[0] + t**3 * d[0],
                u**3 * a[1] + 3 * u * u * t * b[1] + 3 * u * t * t * c[1] + t**3 * d[1],
            )
        )


def densify(points, out, spacing):
    """Adds points along a segment, so distances to edges are seen, not just vertices."""
    a, b = points
    n = max(1, int(math.dist(a, b) / spacing))
    for i in range(1, n + 1):
        t = i / n
        out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))


SPACING = 20_000  # EMU between samples along a straight edge


def outline_from_engine(entry):
    out = []
    for path in entry["paths"]:
        if path["fill"] == "None":
            continue  # stroke-only paths; the deck has no stroke
        cur = (0.0, 0.0)
        start = cur
        for segment in path["segments"]:
            kind = segment[0]
            if kind == "M":
                cur = start = tuple(segment[1])
                out.append(cur)
            elif kind == "L":
                next_point = tuple(segment[1])
                densify((cur, next_point), out, SPACING)
                cur = next_point
            elif kind == "Q":
                c, e = tuple(segment[1]), tuple(segment[2])
                c1 = (cur[0] + 2 / 3 * (c[0] - cur[0]), cur[1] + 2 / 3 * (c[1] - cur[1]))
                c2 = (e[0] + 2 / 3 * (c[0] - e[0]), e[1] + 2 / 3 * (c[1] - e[1]))
                sample_cubic(cur, c1, c2, e, out)
                cur = e
            elif kind == "C":
                b, c, e = tuple(segment[1]), tuple(segment[2]), tuple(segment[3])
                sample_cubic(cur, b, c, e, out)
                cur = e
            elif kind == "Z":
                densify((cur, start), out, SPACING)
                cur = start
    return out


def outline_from_svg(svg_path):
    root = ET.parse(svg_path).getroot()
    SVG = "{http://www.w3.org/2000/svg}"
    out = []
    for group in root.iter(SVG + "g"):
        if group.get("class") != "com.sun.star.drawing.CustomShape":
            continue
        for node in group.iter(SVG + "path"):
            if node.get("fill") in (None, "none"):
                continue
            tokens = re.findall(r"[MLCZz]|-?[0-9.]+", node.get("d"))
            i, cur, start, op = 0, (0.0, 0.0), (0.0, 0.0), None
            while i < len(tokens):
                if tokens[i] in "MLCZz":
                    op = tokens[i]
                    i += 1
                    if op in "Zz":
                        densify((cur, start), out, SPACING / EMU_PER_SVG_UNIT)
                        cur = start
                    continue
                take = 6 if op == "C" else 2
                v = [float(x) for x in tokens[i : i + take]]
                i += take
                if op == "M":
                    cur = start = (v[0], v[1])
                    out.append(cur)
                    op = "L"
                elif op == "L":
                    densify((cur, (v[0], v[1])), out, SPACING / EMU_PER_SVG_UNIT)
                    cur = (v[0], v[1])
                else:
                    sample_cubic(cur, v[:2], v[2:4], v[4:6], out)
                    cur = (v[4], v[5])
    # To shape EMU.
    return [(x * EMU_PER_SVG_UNIT - OFF_X, y * EMU_PER_SVG_UNIT - OFF_Y) for x, y in out]


def to_segment(p, a, b):
    """Squared distance from a point to a segment, not merely to its end points."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    length = dx * dx + dy * dy
    t = 0.0 if length == 0 else max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length))
    x, y = a[0] + t * dx - p[0], a[1] + t * dy - p[1]
    return x * x + y * y


def one_way(a, b):
    """The farthest any sample of `a` lies from the polyline through `b`. Jumps between
    subpaths add stray segments to `b`, which can only hide a gap at a subpath's start,
    so the moves are skipped by treating runs longer than a sample step as breaks."""
    pairs = [(p, q) for p, q in zip(b, b[1:]) if math.dist(p, q) <= 400_000]
    worst = 0.0
    for p in a:
        worst = max(worst, min(to_segment(p, q, r) for q, r in pairs))
    return math.sqrt(worst)


def main():
    if os.environ.get("OPENPRESENT_ALLOW_SOFFICE") != "1":
        raise SystemExit("soffice is off; set OPENPRESENT_ALLOW_SOFFICE=1 to run it on purpose")
    dump, work = sys.argv[1], pathlib.Path(sys.argv[2])
    names = sys.argv[3:] or DEFAULT
    work.mkdir(parents=True, exist_ok=True)
    entries = {}
    for line in pathlib.Path(dump).read_text(encoding="utf8").splitlines():
        entry = json.loads(line)
        entries[entry["name"]] = entry
    for name in names:
        deck = work / f"{name}.pptx"
        write_deck(deck, name)
    subprocess.run(
        [
            "soffice",
            f"-env:UserInstallation=file://{work.resolve()}/profile",
            "--headless",
            "--convert-to",
            "svg",
            "--outdir",
            str(work),
            *[str(work / f"{n}.pptx") for n in names],
        ],
        check=True,
        capture_output=True,
        timeout=600,
    )
    print(f"{'preset':28} {'engine pts':>10} {'LO pts':>7} {'max distance, EMU':>18} {'as % of short side':>18}")
    for name in names:
        ours = outline_from_engine(entries[name])
        theirs = outline_from_svg(work / f"{name}.svg")
        if not theirs:
            print(f"{name:28} no shape found in SVG")
            continue
        distance = max(one_way(ours, theirs), one_way(theirs, ours))
        print(f"{name:28} {len(ours):10} {len(theirs):7} {distance:18.0f} {100 * distance / min(W, H):17.3f}%")


if __name__ == "__main__":
    main()
