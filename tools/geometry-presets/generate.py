#!/usr/bin/env python3
"""Generate the preset geometry tables of crates/casual-slide-geometry.

Reads schemas/geometry/preset{Shape,TextWarp}Definitions.xml and writes
crates/casual-slide-geometry/src/generated.rs: guide programs with every name
already resolved to an evaluation slot, so the crate parses no XML at runtime.

    python3 tools/geometry-presets/generate.py          # rewrite the table
    python3 tools/geometry-presets/generate.py --check  # fail if it is stale

The output depends only on the two definition files, dml-main.xsd and this
script: no timestamps, no hash ordering, presets sorted by name.

Generation fails, rather than guessing, on anything the source files do that is
not listed below. The listed items are defects in the vendored definitions and
each one is corrected here explicitly:

GEO-01  `upDownArrow` appears twice (byte-identical) and `upArrow` never, while
        ST_ShapeType requires it. A dict keyed on the name silently yields 186.
        See UP_ARROW_XML for the derivation.
GEO-02  ST_TextShapeType has 41 values; the warp file defines 40. `textNoShape`
        is absent. It means "no warp", so it gets an empty definition.
GEO-03  `curvedLeftArrow` site 2 has ang="cd3", a guide nothing defines. See
        ATTRIBUTE_FIXES.
GEO-04  Eight formulas in the circular arrows carry a fourth `+-` operand of
        literal 0 (`+- xH 0 dxB 0`). `+-` takes three; see SURPLUS_OPERANDS.
GEO-05  `pie` has <rect l="il" t="ir" r="it" b="ib"/>: top and right exchanged, so
        the text rectangle is inside out. See RECT_FIXES.
GEO-06  `gear6` and `gear9` compute their mirrored connection sites with `+/` where
        `+-` is meant: `xCxn4 = +/ r 0 xCxn1` is r / xCxn1, about 1 EMU at any size,
        so those sites all sit on the left edge. See FORMULA_FIXES.

Not defects, but the evaluator must honour them, so they are pinned in tests:
guide names reused within a preset (latest definition wins from that point on),
`r` rebound by textRingInside and textRingOutside, `hd10` used by two flowchart
presets though it is missing from most copies of the built-in guide list, and
`<close/>` followed by `<lnTo>` in the accent callouts.
"""
import pathlib
import re
import sys
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[2]
SHAPES = ROOT / "schemas/geometry/presetShapeDefinitions.xml"
WARPS = ROOT / "schemas/geometry/presetTextWarpDefinitions.xml"
XSD = ROOT / "schemas/ooxml/dml-main.xsd"
OUT = ROOT / "crates/casual-slide-geometry/src/generated.rs"

A = "{http://schemas.openxmlformats.org/drawingml/2006/main}"
XS = "{http://www.w3.org/2001/XMLSchema}"

# Evaluation slot order. The crate's `Builtins` computes values in this order and
# a test compares it with the copy emitted into the table. Sorted, for bisection.
BUILTINS = sorted(
    "3cd4 3cd8 5cd8 7cd8 b cd2 cd4 cd8 h hc hd10 hd2 hd3 hd4 hd5 hd6 hd8 l ls r ss "
    "ssd16 ssd2 ssd32 ssd4 ssd6 ssd8 t vc w wd10 wd12 wd16 wd2 wd3 wd32 wd4 wd5 wd6 "
    "wd8".split()
)

OPS = {  # formula token: (Rust variant, operand count)
    "val": ("Val", 1),
    "abs": ("Abs", 1),
    "sqrt": ("Sqrt", 1),
    "at2": ("At2", 2),
    "cos": ("Cos", 2),
    "sin": ("Sin", 2),
    "tan": ("Tan", 2),
    "max": ("Max", 2),
    "min": ("Min", 2),
    "*/": ("MulDiv", 3),
    "+-": ("AddSub", 3),
    "+/": ("AddDiv", 3),
    "?:": ("If", 3),
    "cat2": ("Cat2", 3),
    "sat2": ("Sat2", 3),
    "mod": ("Mod", 3),
    "pin": ("Pin", 3),
}

# GEO-01. `upArrow`, derived from `downArrow` in the same file by y -> h - y, with
# these decisions, each of which a test pins:
#
# * Guides. downArrow's `y1 = b - dy1` becomes `y2 = t + dy2`, and its text-rect
#   bottom `y2 = y1 + dy2`, measured from the head into the shaft, becomes
#   `y1 = y2 - dy1`. maxAdj2, a1, a2 and dx1 do not depend on direction.
# * Vertex order. A pure reflection would reverse the winding. Every other arrow
#   here runs clockwise on screen, and upDownArrow's upper head is drawn
#   `l,y2 -> hc,t -> r,y2`, so this does too, starting at the left shoulder. The
#   filled area is unaffected; the dash phase and start point are not.
# * Connection sites. Index is persisted in files (a:stCxn idx), so order matters:
#   the four-site arrows all use top, left, bottom, right (3cd4, cd2, cd4, 0).
# * Adjust handles. adj1 sits at the shaft's far end, now at the bottom.
#
# LibreOffice's independently shipped copy (filter/oox-drawingml-cs-presets)
# agrees on equations, vertex order, glue points, text frame and handles.
UP_ARROW_XML = """
<upArrow xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:avLst><a:gd name="adj1" fmla="val 50000"/><a:gd name="adj2" fmla="val 50000"/></a:avLst>
  <a:gdLst>
    <a:gd name="maxAdj2" fmla="*/ 100000 h ss"/>
    <a:gd name="a1" fmla="pin 0 adj1 100000"/>
    <a:gd name="a2" fmla="pin 0 adj2 maxAdj2"/>
    <a:gd name="dy2" fmla="*/ ss a2 100000"/>
    <a:gd name="y2" fmla="+- t dy2 0"/>
    <a:gd name="dx1" fmla="*/ w a1 200000"/>
    <a:gd name="x1" fmla="+- hc 0 dx1"/>
    <a:gd name="x2" fmla="+- hc dx1 0"/>
    <a:gd name="dy1" fmla="*/ x1 dy2 wd2"/>
    <a:gd name="y1" fmla="+- y2 0 dy1"/>
  </a:gdLst>
  <a:ahLst>
    <a:ahXY gdRefX="adj1" minX="0" maxX="100000"><a:pos x="x1" y="b"/></a:ahXY>
    <a:ahXY gdRefY="adj2" minY="0" maxY="maxAdj2"><a:pos x="l" y="y2"/></a:ahXY>
  </a:ahLst>
  <a:cxnLst>
    <a:cxn ang="3cd4"><a:pos x="hc" y="t"/></a:cxn>
    <a:cxn ang="cd2"><a:pos x="l" y="y2"/></a:cxn>
    <a:cxn ang="cd4"><a:pos x="hc" y="b"/></a:cxn>
    <a:cxn ang="0"><a:pos x="r" y="y2"/></a:cxn>
  </a:cxnLst>
  <a:rect l="x1" t="y1" r="x2" b="b"/>
  <a:pathLst>
    <a:path>
      <a:moveTo><a:pt x="l" y="y2"/></a:moveTo>
      <a:lnTo><a:pt x="hc" y="t"/></a:lnTo>
      <a:lnTo><a:pt x="r" y="y2"/></a:lnTo>
      <a:lnTo><a:pt x="x2" y="y2"/></a:lnTo>
      <a:lnTo><a:pt x="x2" y="b"/></a:lnTo>
      <a:lnTo><a:pt x="x1" y="b"/></a:lnTo>
      <a:lnTo><a:pt x="x1" y="y2"/></a:lnTo>
      <a:close/>
    </a:path>
  </a:pathLst>
</upArrow>
"""

# GEO-02. Present only so the table matches ST_TextShapeType; draws nothing.
TEXT_NO_SHAPE_XML = """
<textNoShape xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:pathLst/>
</textNoShape>
"""

# GEO-03. (preset, site index): (found, used). curvedRightArrow has the mirror-image
# site `r,y6` at ang 0; this one is `l,y6`, on the left edge, facing left, which is
# cd2. `cd3` would be 120 degrees and is not a built-in guide in any list.
ATTRIBUTE_FIXES = {("curvedLeftArrow", 2): ("cd3", "cd2")}

# GEO-05. preset: (found, used), as l t r b. Every other arc-bounded shape (ellipse,
# chord, blockArc, noSmoking) uses il it ir ib, and only this order gives l < r and
# t < b. LibreOffice ships the swapped text as-is. PowerPoint has not been measured,
# so this is the sensible reading, not a confirmed match.
RECT_FIXES = {"pie": (("il", "ir", "it", "ib"), ("il", "it", "ir", "ib"))}

# GEO-06. (preset, guide): (found, used). The site's mirror on the other side is r
# minus its x; every sibling xD5 = +- w 0 xA4 in these same presets is written that
# way. LibreOffice ships the `+/` text as-is, so the two disagree here on purpose.
FORMULA_FIXES = {
    ("gear6", "xCxn4"): ("+/ r 0 xCxn1", "+- r 0 xCxn1"),
    ("gear9", "xCxn5"): ("+/ r 0 xCxn4", "+- r 0 xCxn4"),
    ("gear9", "xCxn6"): ("+/ r 0 xCxn3", "+- r 0 xCxn3"),
    ("gear9", "xCxn7"): ("+/ r 0 xCxn2", "+- r 0 xCxn2"),
    ("gear9", "xCxn8"): ("+/ r 0 xCxn1", "+- r 0 xCxn1"),
}

# GEO-04. Exactly these may carry a surplus operand, which must be literal 0 and
# is dropped: it could only have meant `+ 0`, so the value is not in doubt.
SURPLUS_OPERANDS = {
    ("circularArrow", "xB"),
    ("circularArrow", "yB"),
    ("leftCircularArrow", "xB"),
    ("leftCircularArrow", "yB"),
    ("leftRightCircularArrow", "xB"),
    ("leftRightCircularArrow", "yB"),
    ("leftRightCircularArrow", "xJ"),
    ("leftRightCircularArrow", "yJ"),
}

INTEGER = re.compile(r"-?[0-9]+\Z")


def fail(message):
    sys.exit(f"geometry-presets: {message}")


def schema_values(simple_type):
    root = ET.parse(XSD).getroot()
    for node in root.findall(XS + "simpleType"):
        if node.get("name") == simple_type:
            return [e.get("value") for e in node.iter(XS + "enumeration")]
    fail(f"{simple_type} not in {XSD.name}")


def load(path, simple_type, overrides, allow_duplicate):
    """Returns the definitions by name, reconciled against the schema's value set."""
    found = {}
    for shape in ET.parse(path).getroot():
        if shape.tag not in found:
            found[shape.tag] = shape
        elif shape.tag == allow_duplicate:
            first = ET.tostring(found[shape.tag])
            if ET.tostring(shape).strip() != first.strip():
                fail(f"the two {shape.tag} definitions differ; GEO-01 assumed they match")
        else:
            fail(f"{path.name}: unexpected duplicate {shape.tag}")
    wanted = schema_values(simple_type)
    if len(wanted) != len(set(wanted)):
        fail(f"{simple_type} has repeated values")
    for name, xml in overrides.items():
        if name in found:
            fail(f"{path.name} now defines {name}; remove the override and re-derive")
        found[name] = ET.fromstring(xml)
    if set(found) != set(wanted):
        fail(
            f"{simple_type} mismatch: missing {sorted(set(wanted) - set(found))}, "
            f"extra {sorted(set(found) - set(wanted))}"
        )
    return found


class Scope:
    """Name to slot, rebound on each definition: references see earlier ones only."""

    def __init__(self, preset):
        self.preset = preset
        self.names = {name: slot for slot, name in enumerate(BUILTINS)}
        self.next = len(BUILTINS)

    def define(self, name):
        self.names[name] = self.next
        self.next += 1

    def arg(self, token):
        if INTEGER.match(token):
            return f"L({int(token)})"
        if token not in self.names:
            fail(f"{self.preset}: {token!r} is not defined at this point")
        return f"S({self.names[token]})"

    def opt(self, token):
        return "None" if token is None else f"Some({self.arg(token)})"

    def name(self, token):
        return "None" if token is None else f'Some(B("{token}"))'


def guide(preset, node, scope):
    name, formula = node.get("name"), node.get("fmla")
    fix = FORMULA_FIXES.get((preset, name))
    if fix:
        if formula != fix[0]:
            fail(f"{preset}.{name}: found {formula!r}; revisit the fix")
        formula = fix[1]
    parts = formula.split()
    if parts[0] not in OPS:
        fail(f"{preset}.{name}: unknown operator in {formula!r}")
    variant, arity = OPS[parts[0]]
    operands = parts[1:]
    if len(operands) != arity:
        if (preset, name) not in SURPLUS_OPERANDS or operands[arity:] != ["0"]:
            fail(f"{preset}.{name}: {len(operands)} operands in {formula!r}")
        operands = operands[:arity]
    args = [scope.arg(t) for t in operands] + ["L(0)"] * (3 - arity)
    scope.define(name)
    return f'g("{name}", Op::{variant}, {args[0]}, {args[1]}, {args[2]})'


def point(pt, scope):
    return f"{scope.arg(pt.get('x'))}, {scope.arg(pt.get('y'))}"


def command(node, scope):
    tag = node.tag.removeprefix(A)
    pts = [point(p, scope) for p in node.findall(A + "pt")]
    if tag == "moveTo":
        return f"m({pts[0]})"
    if tag == "lnTo":
        return f"l({pts[0]})"
    if tag == "quadBezTo":
        return f"q({pts[0]}, {pts[1]})"
    if tag == "cubicBezTo":
        return f"c({pts[0]}, {pts[1]}, {pts[2]})"
    if tag == "close":
        return "Cmd::Close"
    if tag == "arcTo":
        values = [scope.arg(node.get(k)) for k in ("wR", "hR", "stAng", "swAng")]
        return f"a({', '.join(values)})"
    fail(f"{scope.preset}: unknown path element {tag}")


def handle(node, scope):
    pos = point(node.find(A + "pos"), scope)
    keys = ("gdRefX", "minX", "maxX", "gdRefY", "minY", "maxY")
    variant = "hxy"
    if node.tag == A + "ahPolar":
        keys = ("gdRefR", "minR", "maxR", "gdRefAng", "minAng", "maxAng")
        variant = "hpolar"
    fields = []
    for i in (0, 3):
        fields.append(scope.name(node.get(keys[i])))
        fields.append(scope.opt(node.get(keys[i + 1])))
        fields.append(scope.opt(node.get(keys[i + 2])))
    return f"{variant}({', '.join(fields)}, {pos})"


FILLS = {
    None: "Norm",
    "norm": "Norm",
    "none": "None",
    "lighten": "Lighten",
    "lightenLess": "LightenLess",
    "darken": "Darken",
    "darkenLess": "DarkenLess",
}


def flag(node, attribute):
    value = node.get(attribute)
    if value not in (None, "true", "false", "0", "1"):
        fail(f"bad boolean {attribute}={value!r}")
    return "false" if value in ("false", "0") else "true"


def emit(preset, node, prefix, out):
    """Appends the statics of one definition and returns its constructor text."""
    scope = Scope(preset)
    sections = {c.tag.removeprefix(A): c for c in node}
    unknown = set(sections) - {"avLst", "gdLst", "ahLst", "cxnLst", "rect", "pathLst"}
    if unknown:
        fail(f"{preset}: unknown sections {sorted(unknown)}")

    def table(suffix, type_name, items):
        if not items:
            return "B(&[])"
        out.append(f"static {prefix}_{suffix}: [{type_name}; {len(items)}] = [")
        out.extend(f"    {item}," for item in items)
        out.append("];")
        return f"B(&{prefix}_{suffix})"

    def guides(section):
        return [guide(preset, g, scope) for g in sections[section]] if section in sections else []

    av = table("AV", "GuideDef", guides("avLst"))
    gd = table("GD", "GuideDef", guides("gdLst"))
    handles = [handle(h, scope) for h in sections.get("ahLst", [])]
    ah = table("AH", "HandleDef", handles)

    sites = []
    for index, site in enumerate(sections.get("cxnLst", [])):
        angle = site.get("ang")
        fix = ATTRIBUTE_FIXES.get((preset, index))
        if fix:
            if angle != fix[0]:
                fail(f"{preset} site {index}: expected {fix[0]}, found {angle}; revisit the fix")
            angle = fix[1]
        sites.append(f"s({scope.arg(angle)}, {point(site.find(A + 'pos'), scope)})")
    cxn = table("CXN", "SiteDef", sites)

    rect = "None"
    if "rect" in sections:
        names = tuple(sections["rect"].get(k) for k in "ltrb")
        if preset in RECT_FIXES:
            if names != RECT_FIXES[preset][0]:
                fail(f"{preset} rect: found {names}; revisit the fix")
            names = RECT_FIXES[preset][1]
        rect = "Some(r(" + ", ".join(scope.arg(name) for name in names) + "))"

    paths = []
    for index, path in enumerate(sections["pathLst"]):
        commands = [command(c, scope) for c in path]
        body = table(f"P{index}", "Cmd", commands)
        for attribute in ("w", "h"):
            if not INTEGER.match(path.get(attribute, "0")):
                fail(f"{preset}: path {attribute} is not a literal")
        paths.append(
            f"PathDef {{ w: {path.get('w', '0')}, h: {path.get('h', '0')}, "
            f"fill: FillMode::{FILLS[path.get('fill')]}, stroke: {flag(path, 'stroke')}, "
            f"extrusion_ok: {flag(path, 'extrusionOk')}, cmds: {body} }}"
        )
    path_table = table("PATHS", "PathDef", paths)

    return (
        f'GeometryDef {{ name: B("{preset}"), av: {av}, gd: {gd}, handles: {ah}, '
        f"sites: {cxn}, rect: {rect}, paths: {path_table} }}"
    )


def render():
    shapes = load(SHAPES, "ST_ShapeType", {"upArrow": UP_ARROW_XML}, "upDownArrow")
    warps = load(WARPS, "ST_TextShapeType", {"textNoShape": TEXT_NO_SHAPE_XML}, None)
    out = [
        "// @generated by tools/geometry-presets/generate.py. Do not edit: run it instead.",
        "// Sources: schemas/geometry/preset{Shape,TextWarp}Definitions.xml, with the",
        "// corrections GEO-01 to GEO-06 described in that script.",
        "",
        "use std::borrow::Cow::Borrowed as B;",
        "",
        "use crate::def::Arg::{Lit as L, Slot as S};",
        "use crate::def::ctor::{a, c, g, hpolar, hxy, l, m, q, r, s};",
        "use crate::def::{Cmd, FillMode, GeometryDef, GuideDef, HandleDef, Op, PathDef, SiteDef};",
        "",
        f"pub(crate) const BUILTIN_ORDER: [&str; {len(BUILTINS)}] = [",
    ]
    out.extend(f'    "{name}",' for name in BUILTINS)
    out.append("];")
    out.append("")
    for prefix, static, definitions in (
        ("S", "PRESET_SHAPES", shapes),
        ("W", "TEXT_WARPS", warps),
    ):
        ctors = []
        for index, name in enumerate(sorted(definitions)):
            ctors.append(emit(name, definitions[name], f"{prefix}{index:03}", out))
        out.append("")
        out.append(f"pub(crate) static {static}: [GeometryDef; {len(ctors)}] = [")
        out.extend(f"    {ctor}," for ctor in ctors)
        out.append("];")
        out.append("")
    return "\n".join(out)


def main():
    text = render()
    if "--check" in sys.argv[1:]:
        current = OUT.read_text(encoding="utf8") if OUT.exists() else ""
        if current != text:
            fail(f"{OUT.relative_to(ROOT)} is stale; run tools/geometry-presets/generate.py")
        print("geometry-presets: table is current (187 shapes, 41 warps)")
        return
    OUT.write_text(text, encoding="utf8")
    print(f"wrote {OUT.relative_to(ROOT)} ({len(text)} bytes)")


if __name__ == "__main__":
    main()
