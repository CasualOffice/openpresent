#!/usr/bin/env python3
"""Decks that show every value of an enumeration, one deck per checklist area where cheap.

Enum values are read from the vendored schemas, so a value cannot be left out by
recollection. Each shape carries its value as a label, so a rendered page is readable.
fixtures/tools/generate.py pulls these into the manifest; run it to write them.

    python3 fixtures/tools/generate_presets.py --validate   # every part through tools/f4/validate.py

Not done here, because they are not cheap or need parts with real content: charts,
SmartArt, equations, ink, media, OLE, comments, VML, signatures, and the extension namespaces.
"""
import pathlib
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
import zipfile
import zlib
from struct import pack
from xml.sax.saxutils import escape

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from generate import DECL, NS, TREE, deck, replace, zipped  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
XS = "{http://www.w3.org/2001/XMLSchema}"
_SCHEMAS = {}
WIDTH, HEIGHT = 12192000, 6858000
MARGIN, TOP = 228600, 640080
A_TABLE = "http://schemas.openxmlformats.org/drawingml/2006/table"


def enum(name, schema="dml-main"):
    """The values of a simple type, in schema order."""
    if schema not in _SCHEMAS:
        _SCHEMAS[schema] = ET.parse(REPO / f"schemas/ooxml/{schema}.xsd").getroot()
    for node in _SCHEMAS[schema].findall(f"{XS}simpleType"):
        if node.get("name") == name:
            values = [e.get("value") for e in node.iter(f"{XS}enumeration")]
            for union in node.iter(f"{XS}union"):
                for member in (union.get("memberTypes") or "").split():
                    values += enum(member.split(":")[-1], schema)
            if not values:
                raise SystemExit(f"{name}: no enumeration in {schema}")
            return values
    raise SystemExit(f"{name}: not in {schema}")


class Slide:
    """Hands out shape ids, so ids stay unique within a slide."""

    def __init__(self, title, attrs="", bg="", tail=""):
        self.next_id, self.body, self.attrs, self.bg, self.tail = 3, [], attrs, bg, tail
        self.title = title

    def nid(self):
        self.next_id += 1
        return self.next_id - 1

    def add(self, xml):
        self.body.append(xml)

    def xml(self):
        title = (f'<p:sp><p:nvSpPr><p:cNvPr id="2" name="Title 1"/><p:cNvSpPr/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>'
                 f'<p:spPr><a:xfrm><a:off x="{MARGIN}" y="91440"/><a:ext cx="{WIDTH - 2 * MARGIN}" cy="457200"/></a:xfrm>'
                 '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
                 f'<p:txBody><a:bodyPr anchor="ctr"/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="2000" b="1"/><a:t>{escape(self.title)}</a:t></a:r></a:p></p:txBody></p:sp>')
        return (f'{DECL}<p:sld {NS}{self.attrs}><p:cSld>{self.bg}<p:spTree>{TREE}{title}{"".join(self.body)}</p:spTree></p:cSld>'
                f'{self.tail}</p:sld>')


def xfrm(x, y, cx, cy, attrs=""):
    return f'<a:xfrm{attrs}><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>'


def geom(prst="rect", av=""):
    return f'<a:prstGeom prst="{prst}"><a:avLst>{av}</a:avLst></a:prstGeom>'


ACCENT = '<a:solidFill><a:schemeClr val="accent1"/></a:solidFill>'
DARK_LINE = '<a:ln w="9525"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill></a:ln>'


def shape(s, box, props=ACCENT + DARK_LINE, prst="rect", text="", body="", name="Shape"):
    """A filled shape; props is everything in spPr after the geometry."""
    sid = s.nid()
    x, y, cx, cy = box
    tx = ""
    if text or body:
        tx = f'<p:txBody>{body or "<a:bodyPr/>"}<a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="900"/><a:t>{escape(text)}</a:t></a:r></a:p></p:txBody>'
    s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{escape(name)} {sid}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
          f'<p:spPr>{xfrm(x, y, cx, cy)}{geom(prst)}{props}</p:spPr>{tx}</p:sp>')
    return sid


def label(s, box, text, sz=900):
    sid = s.nid()
    x, y, cx, cy = box
    s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Label {sid}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>'
          f'<p:spPr>{xfrm(x, y, cx, cy)}{geom()}<a:noFill/></p:spPr>'
          '<p:txBody><a:bodyPr wrap="none" lIns="0" tIns="0" rIns="0" bIns="0" anchor="t"/><a:lstStyle/>'
          f'<a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="en-US" sz="{sz}"/><a:t>{escape(text)}</a:t></a:r></a:p></p:txBody></p:sp>')


def grid(title, items, cols, rows, draw, label_h=182880):
    """Paginate items over slides, cols by rows, each drawn with its value below it."""
    slides = []
    per_page = cols * rows
    cell_w = (WIDTH - 2 * MARGIN) // cols
    cell_h = (HEIGHT - TOP - 91440) // rows
    pad = 45720
    for page in range(0, len(items), per_page):
        chunk = items[page:page + per_page]
        s = Slide(f"{title} ({page + 1}-{page + len(chunk)} of {len(items)})")
        for i, item in enumerate(chunk):
            x = MARGIN + (i % cols) * cell_w
            y = TOP + (i // cols) * cell_h
            box = (x + pad, y + pad, cell_w - 2 * pad, cell_h - 2 * pad - label_h)
            draw(s, box, item)
            label(s, (x, y + cell_h - label_h - pad, cell_w, label_h), str(item if isinstance(item, str) else item[0]))
        slides.append(s)
    return slides


def build(slides, **kwargs):
    """A deck from Slide objects, on generate.py's minimal package."""
    entries = deck(len(slides), **kwargs)
    for i, s in enumerate(slides, 1):
        entries = replace(entries, f"ppt/slides/slide{i}.xml", s.xml())
    return zipped(entries)


def checker_png():
    """8 by 8 black and white, so a tile, crop or stretch is visible. Deterministic."""
    rows = b"".join(b"\0" + bytes(255 if (x // 2 + y // 2) % 2 else 40 for x in range(8)) for y in range(8))
    def chunk(tag, data):
        return pack(">I", len(data)) + tag + data + pack(">I", zlib.crc32(tag + data))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", pack(">IIBBBBB", 8, 8, 8, 0, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


IMAGE = dict(slide_rels=[("image", "../media/image1.png")], parts=[("ppt/media/image1.png", checker_png())])


# ---------------------------------------------------------------- GEOM

def preset_shapes():
    def draw(s, box, prst):
        shape(s, box, prst=prst, name=prst)
    values = enum("ST_ShapeType")
    if len(values) != 187:
        raise SystemExit(f"ST_ShapeType has {len(values)} values, docs/51 says 187")
    return build(grid("Preset shapes, ST_ShapeType", values, 8, 4, draw))


def text_warps():
    def draw(s, box, prst):
        shape(s, box, props="<a:noFill/>", text="Warped text", name=prst,
              body=f'<a:bodyPr><a:prstTxWarp prst="{prst}"><a:avLst/></a:prstTxWarp></a:bodyPr>')
    return build(grid("Text warps, ST_TextShapeType", enum("ST_TextShapeType"), 7, 6, draw))


def custom_geometry():
    s = Slide("Custom geometry: every path command")
    path = ('<a:path w="100" h="100"><a:moveTo><a:pt x="0" y="50"/></a:moveTo><a:lnTo><a:pt x="25" y="0"/></a:lnTo>'
            '<a:arcTo wR="25" hR="25" stAng="10800000" swAng="10800000"/>'
            '<a:quadBezTo><a:pt x="90" y="50"/><a:pt x="75" y="100"/></a:quadBezTo>'
            '<a:cubicBezTo><a:pt x="50" y="80"/><a:pt x="25" y="120"/><a:pt x="0" y="50"/></a:cubicBezTo><a:close/></a:path>')
    cust = ('<a:custGeom><a:avLst><a:gd name="adj" fmla="val 5000"/></a:avLst>'
            '<a:gdLst><a:gd name="g1" fmla="*/ w adj 100000"/></a:gdLst>'
            '<a:ahLst><a:ahXY gdRefX="adj" minX="0" maxX="100000"><a:pos x="g1" y="t"/></a:ahXY></a:ahLst>'
            '<a:cxnLst><a:cxn ang="0"><a:pos x="r" y="vc"/></a:cxn></a:cxnLst><a:rect l="l" t="t" r="r" b="b"/>'
            f'<a:pathLst>{path}</a:pathLst></a:custGeom>')
    sid = s.nid()
    s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Freeform {sid}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
          f'<p:spPr>{xfrm(2286000, 1371600, 4572000, 4114800)}{cust}{ACCENT}{DARK_LINE}</p:spPr></p:sp>')
    return build([s])


# ---------------------------------------------------------------- COLOR

def colors():
    slides = []

    def scheme(s, box, v):
        shape(s, box, props=f'<a:solidFill><a:schemeClr val="{v}"/></a:solidFill>', name=v)

    def preset(s, box, v):
        shape(s, box, props=f'<a:solidFill><a:prstClr val="{v}"/></a:solidFill>', name=v)

    def system(s, box, v):
        shape(s, box, props=f'<a:solidFill><a:sysClr val="{v}" lastClr="808080"/></a:solidFill>', name=v)

    def transform(s, box, item):
        shape(s, box, props=f'<a:solidFill><a:srgbClr val="4472C4">{item[1]}</a:srgbClr></a:solidFill>', name=item[0])

    plain = ("comp", "inv", "gray", "gamma", "invGamma")
    pct = ("tint", "shade", "alpha", "alphaOff", "alphaMod", "hueMod", "sat", "satOff", "satMod", "lum", "lumOff", "lumMod",
           "red", "redOff", "redMod", "green", "greenOff", "greenMod", "blue", "blueOff", "blueMod")
    moves = [(n, f"<a:{n}/>") for n in plain] + [(n, f'<a:{n} val="50000"/>') for n in pct]
    moves += [("hue", '<a:hue val="5400000"/>'), ("hueOff", '<a:hueOff val="5400000"/>')]
    models = [("hslClr", '<a:hslClr hue="5400000" sat="50000" lum="50000"/>'), ("scrgbClr", '<a:scrgbClr r="50000" g="20000" b="80000"/>')]

    def model(s, box, item):
        shape(s, box, props=f"<a:solidFill>{item[1]}</a:solidFill>", name=item[0])

    slides += grid("Scheme colours, ST_SchemeColorVal", enum("ST_SchemeColorVal"), 6, 4, scheme)
    slides += grid("Preset colours, ST_PresetColorVal", enum("ST_PresetColorVal"), 10, 6, preset)
    slides += grid("System colours, ST_SystemColorVal", enum("ST_SystemColorVal"), 6, 5, system)
    slides += grid("Colour transforms on 4472C4", moves, 7, 4, transform)
    slides += grid("Colour models", models, 4, 2, model)
    return build(slides)


# ---------------------------------------------------------------- FILL

def fills():
    slides = []

    def pattern(s, box, v):
        shape(s, box, props=f'<a:pattFill prst="{v}"><a:fgClr><a:srgbClr val="1F3864"/></a:fgClr><a:bgClr><a:srgbClr val="FFFFFF"/></a:bgClr></a:pattFill>', name=v)

    stops = '<a:gsLst><a:gs pos="0"><a:srgbClr val="C00000"/></a:gs><a:gs pos="50000"><a:srgbClr val="FFC000"/></a:gs><a:gs pos="100000"><a:srgbClr val="1F3864"/></a:gs></a:gsLst>'
    grads = [("lin 0", f'<a:lin ang="0" scaled="0"/>'), ("lin 45 scaled", '<a:lin ang="2700000" scaled="1"/>')]
    grads += [(f"path {v}", f'<a:path path="{v}"><a:fillToRect l="50000" t="50000" r="50000" b="50000"/></a:path>') for v in enum("ST_PathShadeType")]
    flips = enum("ST_TileFlipMode")

    def gradient(s, box, item):
        shape(s, box, props=f'<a:gradFill rotWithShape="1">{stops}{item[1]}</a:gradFill>', name=item[0])

    def blip(mode):
        return f'<a:blipFill dpi="0" rotWithShape="1"><a:blip r:embed="rId2"/><a:srcRect/>{mode}</a:blipFill>'

    def picture(s, box, item):
        shape(s, box, props=blip(item[1]), name=item[0])

    aligns = enum("ST_RectAlignment")
    tiles = [("stretch", '<a:stretch><a:fillRect/></a:stretch>')]
    tiles += [(f"tile flip {v}", f'<a:tile tx="0" ty="0" sx="200000" sy="200000" flip="{v}" algn="tl"/>') for v in flips]
    tiles += [(f"tile algn {v}", f'<a:tile sx="300000" sy="300000" algn="{v}"/>') for v in aligns]

    slides += grid("Pattern fills, ST_PresetPatternVal", enum("ST_PresetPatternVal"), 9, 6, pattern)
    slides += grid("Gradient fills", grads, 3, 2, gradient)
    slides += grid("Picture fills: stretch and tile", tiles, 5, 3, picture)
    s = Slide("No fill and group fill")
    shape(s, (914400, 1143000, 2286000, 1828800), props=f"<a:noFill/>{DARK_LINE}", text="noFill")
    slides.append(s)
    return build(slides, **IMAGE)


# ---------------------------------------------------------------- LINE

def lines():
    slides = []

    def line(s, box, ln):
        x, y, cx, cy = box
        shape(s, (x, y + cy // 2, cx, 0), props=ln, prst="line")

    def dash(s, box, v):
        line(s, box, f'<a:ln w="28575"><a:solidFill><a:srgbClr val="1F3864"/></a:solidFill><a:prstDash val="{v}"/></a:ln>')

    def item_line(s, box, item):
        line(s, box, item[1])

    black = '<a:solidFill><a:srgbClr val="1F3864"/></a:solidFill>'
    cases = [(f"cap {v}", f'<a:ln w="114300" cap="{v}">{black}</a:ln>') for v in enum("ST_LineCap")]
    cases += [(f"cmpd {v}", f'<a:ln w="114300" cmpd="{v}">{black}</a:ln>') for v in enum("ST_CompoundLine")]
    cases += [(f"algn {v}", f'<a:ln w="57150" algn="{v}">{black}</a:ln>') for v in enum("ST_PenAlignment")]
    cases += [("custDash", f'<a:ln w="28575">{black}<a:custDash><a:ds d="300000" sp="100000"/><a:ds d="100000" sp="100000"/></a:custDash></a:ln>')]
    for end in enum("ST_LineEndType"):
        for w, ln in zip(enum("ST_LineEndWidth"), enum("ST_LineEndLength")):
            cases.append((f"{end} {w} {ln}", f'<a:ln w="19050">{black}<a:headEnd type="{end}" w="{w}" len="{ln}"/><a:tailEnd type="{end}" w="{w}" len="{ln}"/></a:ln>'))
    joins = [("round", "<a:round/>"), ("bevel", "<a:bevel/>"), ("miter", '<a:miter lim="800000"/>')]

    def joined(s, box, item):
        shape(s, box, props=f'<a:noFill/><a:ln w="76200">{black}{item[1]}</a:ln>', prst="triangle")

    slides += grid("Line dashes, ST_PresetLineDashVal", enum("ST_PresetLineDashVal"), 4, 3, dash)
    slides += grid("Line properties", cases, 6, 6, item_line)
    slides += grid("Line joins", joins, 3, 1, joined)
    return build(slides)


# ---------------------------------------------------------------- EFFECT

def effects():
    grey = '<a:srgbClr val="000000"><a:alpha val="50000"/></a:srgbClr>'
    gold = '<a:srgbClr val="FFC000"><a:alpha val="60000"/></a:srgbClr>'
    named = [
        ("outerShdw", f'<a:outerShdw blurRad="50800" dist="88900" dir="2700000" sx="102000" sy="102000" rotWithShape="0" algn="ctr">{grey}</a:outerShdw>'),
        ("innerShdw", f'<a:innerShdw blurRad="63500" dist="50800" dir="2700000">{grey}</a:innerShdw>'),
        ("glow", f'<a:glow rad="139700">{gold}</a:glow>'),
        ("softEdge", '<a:softEdge rad="127000"/>'),
        ("reflection", '<a:reflection blurRad="6350" stA="50000" endA="300" endPos="50000" dist="50800" dir="5400000" sy="-100000" algn="bl" rotWithShape="0"/>'),
        ("blur", '<a:blur rad="88900" grow="1"/>'),
    ]
    named += [(f"prstShdw {v}", f'<a:prstShdw prst="{v}" dist="88900" dir="2700000">{grey}</a:prstShdw>') for v in enum("ST_PresetShadowVal")]
    named += [(f"fillOverlay {v}", f'<a:fillOverlay blend="{v}"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:fillOverlay>') for v in enum("ST_BlendMode")]
    named += [(f"shadow algn {v}", f'<a:outerShdw dist="88900" dir="2700000" algn="{v}">{grey}</a:outerShdw>') for v in enum("ST_RectAlignment")]

    def draw(s, box, item):
        shape(s, box, props=f"{ACCENT}{DARK_LINE}<a:effectLst>{item[1]}</a:effectLst>", name=item[0])

    return build(grid("Effects", named, 7, 4, draw))


# ---------------------------------------------------------------- D3

def scene():
    cases = [(f"camera {v}", f'<a:scene3d><a:camera prst="{v}"/><a:lightRig rig="threePt" dir="t"/></a:scene3d>') for v in enum("ST_PresetCameraType")]
    cases += [(f"rig {v}", f'<a:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="{v}" dir="t"/></a:scene3d>') for v in enum("ST_LightRigType")]
    cases += [(f"rig dir {v}", f'<a:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="{v}"/></a:scene3d>') for v in enum("ST_LightRigDirection")]
    plain = '<a:scene3d><a:camera prst="isometricLeftDown"/><a:lightRig rig="threePt" dir="t"/></a:scene3d>'
    cases += [(f"bevel {v}", f'{plain}<a:sp3d extrusionH="76200"><a:bevelT prst="{v}"/><a:bevelB prst="{v}"/></a:sp3d>') for v in enum("ST_BevelPresetType")]
    cases += [(f"material {v}", f'{plain}<a:sp3d extrusionH="76200" prstMaterial="{v}"/>') for v in enum("ST_PresetMaterialType")]

    def draw(s, box, item):
        shape(s, box, props=f"{ACCENT}{DARK_LINE}{item[1]}", name=item[0])

    return build(grid("3D scene and shape", cases, 8, 5, draw))


# ---------------------------------------------------------------- TEXT

def texts():
    def para(ppr="", rpr="", text="Sample text"):
        return f'<a:p><a:pPr{ppr}/><a:r><a:rPr lang="en-US" sz="1000"{rpr}/><a:t>{escape(text)}</a:t></a:r></a:p>'

    def body(bodypr="", paras=None):
        return f"<a:bodyPr{bodypr}/><a:lstStyle/>" + (paras or para())

    cases = []
    cases += [(f"algn {v}", body(paras=para(f' algn="{v}"', text="Two words wide enough to justify when it wraps over lines"))) for v in enum("ST_TextAlignType")]
    cases += [(f"fontAlgn {v}", body(paras=para(f' fontAlgn="{v}"'))) for v in enum("ST_TextFontAlignType")]
    # just and dist are in Ecma's list and the Open XML SDK refuses them: text_ecma_only.
    cases += [(f"anchor {v}", body(f' anchor="{v}"')) for v in enum("ST_TextAnchoringType") if v not in ("just", "dist")]
    cases += [(f"vert {v}", body(f' vert="{v}"')) for v in enum("ST_TextVerticalType")]
    cases += [(f"wrap {v}", body(f' wrap="{v}"', para(text="A long run that either wraps in the box or does not wrap at all"))) for v in enum("ST_TextWrappingType")]
    cases += [(f"vertOverflow {v}", body(f' vertOverflow="{v}"')) for v in enum("ST_TextVertOverflowType")]
    cases += [(f"horzOverflow {v}", body(f' horzOverflow="{v}"')) for v in enum("ST_TextHorzOverflowType")]
    cases += [(f"u {v}", body(paras=para(rpr=f' u="{v}"'))) for v in enum("ST_TextUnderlineType")]
    cases += [(f"strike {v}", body(paras=para(rpr=f' strike="{v}"'))) for v in enum("ST_TextStrikeType")]
    cases += [(f"cap {v}", body(paras=para(rpr=f' cap="{v}"'))) for v in enum("ST_TextCapsType")]
    for v in enum("ST_TextAutonumberScheme"):
        p = (f'<a:pPr marL="228600" indent="-228600"><a:buFont typeface="+mj-lt"/><a:buAutoNum type="{v}" startAt="2"/></a:pPr>'
             '<a:r><a:rPr lang="en-US" sz="1000"/><a:t>Item</a:t></a:r>')
        cases.append((f"buAutoNum {v}", body(paras=f"<a:p>{p}</a:p><a:p>{p}</a:p>")))
    rich = ('<a:p><a:pPr marL="114300" indent="-114300" algn="l" defTabSz="457200" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">'
            '<a:lnSpc><a:spcPct val="120000"/></a:lnSpc><a:spcBef><a:spcPts val="600"/></a:spcBef><a:spcAft><a:spcPts val="300"/></a:spcAft>'
            '<a:buClr><a:srgbClr val="C00000"/></a:buClr><a:buSzPct val="130000"/><a:buFont typeface="Arial"/><a:buChar char="&#8226;"/>'
            '<a:tabLst><a:tab pos="914400" algn="ctr"/></a:tabLst></a:pPr>'
            '<a:r><a:rPr lang="en-US" sz="1000" b="1" i="1" kern="1200" baseline="30000" spc="100">'
            '<a:solidFill><a:srgbClr val="C00000"/></a:solidFill><a:latin typeface="+mn-lt"/></a:rPr><a:t>Rich</a:t></a:r>'
            '<a:br><a:rPr lang="en-US" sz="1000"/></a:br>'
            '<a:fld id="{00000000-0000-4000-8000-000000000001}" type="slidenum"><a:rPr lang="en-US" sz="1000"/><a:t>1</a:t></a:fld>'
            '<a:endParaRPr lang="en-US" sz="1000"/></a:p>')
    cases += [("paragraph properties", f"<a:bodyPr/><a:lstStyle/>{rich}"),
              ("noAutofit", "<a:bodyPr><a:noAutofit/></a:bodyPr><a:lstStyle/>" + para()),
              ("normAutofit", '<a:bodyPr><a:normAutofit fontScale="62500" lnSpcReduction="20000"/></a:bodyPr><a:lstStyle/>' + para()),
              ("spAutoFit", "<a:bodyPr><a:spAutoFit/></a:bodyPr><a:lstStyle/>" + para()),
              ("columns and insets", '<a:bodyPr numCol="2" spcCol="91440" lIns="45720" tIns="45720" rIns="45720" bIns="45720" rot="0" upright="1"/><a:lstStyle/>' + para()),
              ("buNone and buBlip", '<a:bodyPr/><a:lstStyle/><a:p><a:pPr marL="114300"><a:buNone/></a:pPr><a:r><a:rPr lang="en-US" sz="1000"/><a:t>None</a:t></a:r></a:p>'
                                    '<a:p><a:pPr marL="114300"><a:buBlip><a:blip r:embed="rId2"/></a:buBlip></a:pPr><a:r><a:rPr lang="en-US" sz="1000"/><a:t>Picture</a:t></a:r></a:p>')]

    def draw(s, box, item):
        x, y, cx, cy = box
        sid = s.nid()
        s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Text {sid}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>'
              f'<p:spPr>{xfrm(x, y, cx, cy)}{geom()}<a:ln w="6350"><a:solidFill><a:srgbClr val="A5A5A5"/></a:solidFill></a:ln></p:spPr>'
              f'<p:txBody>{item[1]}</p:txBody></p:sp>')

    return build(grid("Text properties", cases, 7, 5, draw), **IMAGE)


def text_font_effects():
    """PAR-67: strikethrough, superscript, subscript and character spacing at a readable
    size, each beside the same text without it, so a rendered page compares them."""
    def run(text, rpr=""):
        return f'<a:r><a:rPr lang="en-US" sz="2800"{rpr}/><a:t>{escape(text)}</a:t></a:r>'

    cases = [("plain", run("Struck text")),
             ("strike sngStrike", run("Struck text", ' strike="sngStrike"')),
             ("strike dblStrike", run("Struck text", ' strike="dblStrike"')),
             ("baseline 30000", run("E = mc") + run("2", ' baseline="30000"')),
             ("baseline -25000", run("H") + run("2", ' baseline="-25000"') + run("O")),
             ("baseline 30000 and -25000", run("x") + run("n", ' baseline="30000"') + run("i", ' baseline="-25000"')),
             ("spc 600", run("Spaced", ' spc="600"')),
             ("spc -150", run("Condensed", ' spc="-150"')),
             ("spc 0", run("Spaced"))]

    def draw(s, box, item):
        x, y, cx, cy = box
        sid = s.nid()
        s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Text {sid}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>'
              f'<p:spPr>{xfrm(x, y, cx, cy)}{geom()}<a:noFill/></p:spPr>'
              f'<p:txBody><a:bodyPr wrap="none" anchor="ctr"/><a:lstStyle/><a:p><a:pPr algn="ctr"/>{item[1]}</a:p></p:txBody></p:sp>')

    return build(grid("Strikethrough, scripts and spacing", cases, 3, 3, draw))


def text_ecma_only():
    """What Ecma's lists allow, F4 accepts, the Open XML SDK refuses and none of the real
    decks writes, kept apart so area-text is what both validators accept (F4-02)."""
    run = '<a:r><a:rPr lang="en-US" sz="1000"/><a:t>Sample text</a:t></a:r>'
    cases = [(f"anchor {v}", f'<a:bodyPr anchor="{v}"/><a:lstStyle/><a:p>{run}</a:p>') for v in ("just", "dist")]
    cases.append(("buSzPct percent string", f'<a:bodyPr/><a:lstStyle/><a:p><a:pPr marL="114300"><a:buSzPct val="100%"/>'
                                            f'<a:buChar char="&#8226;"/></a:pPr>{run}</a:p>'))

    def draw(s, box, item):
        x, y, cx, cy = box
        sid = s.nid()
        s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{item[0]}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>'
              f'<p:spPr>{xfrm(x, y, cx, cy)}{geom()}</p:spPr><p:txBody>{item[1]}</p:txBody></p:sp>')

    return build(grid("Text values only Ecma's lists have", cases, 3, 1, draw))


# ---------------------------------------------------------------- TABLE, PIC, SHAPE, SLIDE, TIMING, TRANS

def table():
    s = Slide("Table: merges, borders, margins, anchors")
    border = '<a:solidFill><a:srgbClr val="1F3864"/></a:solidFill><a:prstDash val="solid"/>'
    lines = "".join(f'<a:{n} w="12700" cap="flat" cmpd="sng" algn="ctr">{border}</a:{n}>' for n in ("lnL", "lnR", "lnT", "lnB"))
    diag = "".join(f'<a:{n} w="6350">{border}</a:{n}>' for n in ("lnTlToBr", "lnBlToTr"))

    def tc(text, attrs="", props=""):
        run = f'<a:r><a:rPr lang="en-US" sz="1200"/><a:t>{escape(text)}</a:t></a:r>' if text else ""
        return f'<a:tc{attrs}><a:txBody><a:bodyPr/><a:lstStyle/><a:p>{run}</a:p></a:txBody>{props}</a:tc>'

    fill = '<a:solidFill><a:srgbClr val="DEEAF6"/></a:solidFill>'
    rows = [
        tc("gridSpan 2", ' gridSpan="2"', f"<a:tcPr>{lines}{fill}</a:tcPr>") + tc("", ' hMerge="1"') + tc("rowSpan 2", ' rowSpan="2"', f'<a:tcPr anchor="ctr">{lines}</a:tcPr>'),
        tc("margins", "", f'<a:tcPr marL="182880" marR="182880" marT="91440" marB="91440">{lines}</a:tcPr>') + tc("diagonals", "", f"<a:tcPr>{lines}{diag}</a:tcPr>") + tc("", ' vMerge="1"'),
    ]
    for v in enum("ST_TextAnchoringType")[:3]:
        rows.append(tc(f"anchor {v}", "", f'<a:tcPr anchor="{v}">{lines}</a:tcPr>') + tc("vert vert270", "", f'<a:tcPr vert="vert270">{lines}</a:tcPr>') + tc("noFill", "", f"<a:tcPr>{lines}<a:noFill/></a:tcPr>"))
    body = "".join(f'<a:tr h="731520">{r}</a:tr>' for r in rows)
    grid_cols = "".join('<a:gridCol w="3048000"/>' for _ in range(3))
    tbl = (f'<a:tbl><a:tblPr firstRow="1" bandRow="1" firstCol="0" lastRow="0" lastCol="0" bandCol="0"/><a:tblGrid>{grid_cols}</a:tblGrid>{body}</a:tbl>')
    sid = s.nid()
    s.add(f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="Table {sid}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>'
          f'<p:xfrm><a:off x="1524000" y="914400"/><a:ext cx="9144000" cy="3657600"/></p:xfrm>'
          f'<a:graphic><a:graphicData uri="{A_TABLE}">{tbl}</a:graphicData></a:graphic></p:graphicFrame>')
    return build([s])


def pictures():
    s = Slide("Pictures: crop, over-crop, compression, flip, rotation")
    cases = [("plain", '<a:srcRect/>', ""), ("crop", '<a:srcRect l="25000" t="25000" r="25000" b="25000"/>', ""),
             ("beyond bounds", '<a:srcRect l="-25000" t="-25000" r="-25000" b="-25000"/>', ""),
             ("rot flipH", "<a:srcRect/>", ' rot="1800000" flipH="1"'), ("flipV", "<a:srcRect/>", ' flipV="1"')]
    for i, (name, crop, attrs) in enumerate(cases):
        x = MARGIN + i * 2286000
        sid = s.nid()
        s.add(f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="{name}" descr="{name}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr>'
              f'<p:blipFill><a:blip r:embed="rId2" cstate="print"/>{crop}<a:stretch><a:fillRect/></a:stretch></p:blipFill>'
              f'<p:spPr>{xfrm(x, 1371600, 1828800, 1828800, attrs)}{geom()}{DARK_LINE}</p:spPr></p:pic>')
        label(s, (x, 3429000, 1828800, 182880), name)
    return build([s], **IMAGE)


def shape_tree():
    s = Slide("Shape tree: nested groups, scale, flips, connectors, locks")
    def inner(sid, x, y):
        return (f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Child {sid}"/><p:cNvSpPr><a:spLocks noMove="1" noResize="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>'
                f'<p:spPr>{xfrm(x, y, 914400, 914400)}{geom("rightArrow")}{ACCENT}{DARK_LINE}</p:spPr>'
                '<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="900"/><a:t>F</a:t></a:r></a:p></p:txBody></p:sp>')
    g, g2, c1, c2 = s.nid(), s.nid(), s.nid(), s.nid()
    group2 = (f'<p:grpSp><p:nvGrpSpPr><p:cNvPr id="{g2}" name="Inner group"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>'
              f'<p:grpSpPr><a:xfrm rot="2700000" flipH="1"><a:off x="914400" y="0"/><a:ext cx="914400" cy="914400"/><a:chOff x="0" y="0"/><a:chExt cx="914400" cy="914400"/></a:xfrm></p:grpSpPr>'
              f'{inner(c2, 0, 0)}</p:grpSp>')
    s.add(f'<p:grpSp><p:nvGrpSpPr><p:cNvPr id="{g}" name="Outer group"/><p:cNvGrpSpPr><a:grpSpLocks noUngrp="1"/></p:cNvGrpSpPr><p:nvPr/></p:nvGrpSpPr>'
          f'<p:grpSpPr><a:xfrm flipV="1"><a:off x="914400" y="1371600"/><a:ext cx="3657600" cy="1828800"/><a:chOff x="0" y="0"/><a:chExt cx="1828800" cy="914400"/></a:xfrm></p:grpSpPr>'
          f'{inner(c1, 0, 0)}{group2}</p:grpSp>')
    a, b, cxn = s.nid(), s.nid(), s.nid()
    for sid, y in ((a, 3657600), (b, 5029200)):
        s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="Node {sid}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{xfrm(6858000, y, 1371600, 685800)}{geom("ellipse")}{ACCENT}</p:spPr></p:sp>')
    s.add(f'<p:cxnSp><p:nvCxnSpPr><p:cNvPr id="{cxn}" name="Connector {cxn}"/><p:cNvCxnSpPr><a:stCxn id="{a}" idx="4"/><a:endCxn id="{b}" idx="0"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr>'
          f'<p:spPr>{xfrm(7543800, 4343400, 0, 685800)}{geom("straightConnector1")}<a:ln w="19050"><a:solidFill><a:srgbClr val="C00000"/></a:solidFill><a:tailEnd type="triangle"/></a:ln></p:spPr></p:cxnSp>')
    hidden = s.nid()
    s.add(f'<p:sp><p:nvSpPr><p:cNvPr id="{hidden}" name="Hidden {hidden}" hidden="1"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{xfrm(0, 0, 914400, 914400)}{geom()}{ACCENT}</p:spPr></p:sp>')
    return build([s])


def slide_props():
    bg_pr = '<p:bg><p:bgPr><a:solidFill><a:srgbClr val="DEEAF6"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>'
    bg_ref = '<p:bg><p:bgRef idx="1001"><a:schemeClr val="accent4"/></p:bgRef></p:bg>'
    over = '<p:clrMapOvr><a:overrideClrMapping bg1="dk1" tx1="lt1" bg2="dk2" tx2="lt2" accent1="accent2" accent2="accent1" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/></p:clrMapOvr>'
    plain = "<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>"
    slides = [Slide("bgPr: slide background", bg=bg_pr, tail=plain), Slide("bgRef: theme background", bg=bg_ref),
              Slide("overrideClrMapping: swapped", tail=over), Slide("Hidden slide, show=0", ' show="0"'),
              Slide("showMasterSp=0", ' showMasterSp="0"')]
    for s in slides:
        shape(s, (914400, 1143000, 2286000, 1371600), text="accent1")
    return build(slides)


def timing():
    s = Slide("Timing: one entrance on click")
    target = shape(s, (914400, 1143000, 2286000, 1371600), text="Appears on click")
    cond0 = '<p:stCondLst><p:cond delay="0"/></p:stCondLst>'
    tgt = '<p:tgtEl><p:sldTgt/></p:tgtEl>'
    effect = (f'<p:par><p:cTn id="5" presetID="1" presetClass="entr" presetSubtype="0" fill="hold" grpId="0" nodeType="clickEffect">{cond0}<p:childTnLst>'
              f'<p:set><p:cBhvr><p:cTn id="6" dur="1" fill="hold">{cond0}</p:cTn><p:tgtEl><p:spTgt spid="{target}"/></p:tgtEl>'
              '<p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="visible"/></p:to></p:set>'
              '</p:childTnLst></p:cTn></p:par>')
    click = (f'<p:par><p:cTn id="3" fill="hold"><p:stCondLst><p:cond delay="indefinite"/></p:stCondLst><p:childTnLst>'
             f'<p:par><p:cTn id="4" fill="hold">{cond0}<p:childTnLst>{effect}</p:childTnLst></p:cTn></p:par></p:childTnLst></p:cTn></p:par>')
    seq = (f'<p:seq concurrent="1" nextAc="seek"><p:cTn id="2" dur="indefinite" nodeType="mainSeq"><p:childTnLst>{click}</p:childTnLst></p:cTn>'
           f'<p:prevCondLst><p:cond evt="onPrev" delay="0">{tgt}</p:cond></p:prevCondLst><p:nextCondLst><p:cond evt="onNext" delay="0">{tgt}</p:cond></p:nextCondLst></p:seq>')
    s.tail = (f'<p:timing><p:tnLst><p:par><p:cTn id="1" dur="indefinite" restart="never" nodeType="tmRoot"><p:childTnLst>{seq}</p:childTnLst></p:cTn></p:par></p:tnLst>'
              f'<p:bldLst><p:bldP spid="{target}" grpId="0"/></p:bldLst></p:timing>')
    return build([s])


def transitions():
    enum_p = lambda n: enum(n, "pml")  # noqa: E731
    cases = []
    for name in ("circle", "dissolve", "diamond", "newsflash", "plus", "random", "wedge"):
        cases.append((name, f"<p:{name}/>"))
    for name in ("blinds", "checker", "comb", "randomBar"):
        cases += [(f"{name} {v}", f'<p:{name} dir="{v}"/>') for v in enum("ST_Direction", "pml")]
    for name in ("cover", "pull"):
        cases += [(f"{name} {v}", f'<p:{name} dir="{v}"/>') for v in enum_p("ST_TransitionEightDirectionType")]
    for name in ("push", "wipe"):
        cases += [(f"{name} {v}", f'<p:{name} dir="{v}"/>') for v in enum_p("ST_TransitionSideDirectionType")]
    cases += [(f"strips {v}", f'<p:strips dir="{v}"/>') for v in enum_p("ST_TransitionCornerDirectionType")]
    cases += [(f"zoom {v}", f'<p:zoom dir="{v}"/>') for v in enum_p("ST_TransitionInOutDirectionType")]
    cases += [("cut thruBlk", '<p:cut thruBlk="1"/>'), ("fade thruBlk", '<p:fade thruBlk="1"/>'), ("wheel 4", '<p:wheel spokes="4"/>')]
    cases += [(f"split {o}", f'<p:split orient="{o}" dir="in"/>') for o in enum("ST_Direction", "pml")]
    speeds = enum_p("ST_TransitionSpeed")
    slides = []
    for i, (name, xml) in enumerate(cases):
        speed = speeds[i % len(speeds)]
        slides.append(Slide(f"{name}, spd {speed}", tail=f'<p:transition spd="{speed}" advClick="1" advTm="{1000 + i}">{xml}</p:transition>'))
    return build(slides)


def fixtures():
    """(relative path, bytes, manifest expectation), in a fixed order."""
    decks = (("preset-shapes", preset_shapes), ("preset-text-warps", text_warps), ("area-custom-geometry", custom_geometry),
             ("area-color", colors), ("area-fill", fills), ("area-line", lines), ("area-effect", effects), ("area-d3", scene),
             ("area-text", texts), ("area-text-ecma-only", text_ecma_only), ("text-font-effects", text_font_effects), ("area-table", table), ("area-pic", pictures), ("area-shape-tree", shape_tree),
             ("area-slide", slide_props), ("area-timing", timing), ("area-trans", transitions))
    for name, make in decks:
        yield f"generated/{name}.pptx", make(), "open"


def validate():
    """Every XML part of every deck here, through F4. Returns its exit code."""
    with tempfile.TemporaryDirectory() as tmp:
        for rel, data, _ in fixtures():
            (pathlib.Path(tmp) / pathlib.Path(rel).name).write_bytes(data)
        return subprocess.run([sys.executable, "-I", str(REPO / "tools/f4/validate.py"), tmp], check=False).returncode


if __name__ == "__main__":
    if "--validate" in sys.argv:
        sys.exit(1 if validate() else 0)
    print(__doc__)
