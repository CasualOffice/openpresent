#!/usr/bin/env python3
"""CLOSE-01 fixtures, for what Tiers 1E and 1F had no deck for: gradient and pattern fills on
placed shapes with text, a body's warp and 3D, a theme's object defaults, `p:style`, the
`extLst` slots, every path fill mode, and the older `a14` namespace. **Synthetic:** written to
the schema, not by PowerPoint, so they show we follow the schema and keep what is there, not
what Office does with it.

Imported by generate.py, which writes them and lists them in the manifest. The expected values
are in crates/casual-slide-import/tests/drawing.rs, crates/casual-slide-export/tests/drawing.rs
and crates/casual-slide-style/tests/text_chain.rs.
"""
import csv
import pathlib

from generate import DECL, NS, THEME, TREE, deck, replace, zipped

REPO = pathlib.Path(__file__).resolve().parents[2]
A14_OLD = "http://schemas.microsoft.com/office/drawing/2007/7/7/main"
ASK = "http://schemas.microsoft.com/office/drawing/2018/sketchyshapes"
FOREIGN_EXT = '<a:extLst><a:ext uri="{7A9C1D30-5B1E-4F7A-8C2D-0E1F2A3B4C5D}"><f:future xmlns:f="urn:example:future"/></a:ext></a:extLst>'
RECT = '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom>'
RED = '<a:srgbClr val="C00000"/>'
BLUE = '<a:schemeClr val="accent1"><a:lumMod val="75000"/></a:schemeClr>'


def enum_values(simple):
    with (REPO / "docs/spec/checklist.tsv").open(encoding="utf-8", newline="") as f:
        return [row["id"].rsplit("/=", 1)[1] for row in csv.DictReader(f, delimiter="\t")
                if row["id"].startswith(f"a:{simple}/=")]


def text_body(words, body="<a:bodyPr/>"):
    return f'<p:txBody>{body}<a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>{words}</a:t></a:r></a:p></p:txBody>'


def placed(sid, name, props, body=None, style="", at=None):
    column, row = at if at is not None else (sid % 8, sid // 8)
    x, y = 150000 + column * 1480000, 100000 + row * 700000
    body = text_body(name) if body is None else body
    return (f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
            f'<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="1400000" cy="620000"/></a:xfrm>{props}</p:spPr>'
            f'{style}{body}</p:sp>')


def slide_of(*shapes):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


def stops(*colors):
    last = max(len(colors) - 1, 1)
    return "".join(f'<a:gs pos="{i * 100000 // last}">{c}</a:gs>' for i, c in enumerate(colors))


def fills_deck():
    """Each gradient kind and each pattern, as a shape on a slide with its own text: the real
    decks in fixtures/real hold neither on a shape, only in themes."""
    gradients = [
        ("Linear down", f'<a:gradFill rotWithShape="1"><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:lin ang="5400000" scaled="0"/></a:gradFill>'),
        ("Linear scaled", f'<a:gradFill flip="none"><a:gsLst>{stops(RED, BLUE, RED)}</a:gsLst><a:lin ang="2700000" scaled="1"/></a:gradFill>'),
        ("Flip x", f'<a:gradFill flip="x" rotWithShape="0"><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:lin ang="0" scaled="0"/>'
                   '<a:tileRect l="-10000" t="0" r="-10000" b="0"/></a:gradFill>'),
        ("Flip y", f'<a:gradFill flip="y"><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:lin ang="0" scaled="0"/><a:tileRect/></a:gradFill>'),
        ("Flip both", f'<a:gradFill flip="xy"><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:lin ang="10800000" scaled="0"/></a:gradFill>'),
        ("Radial", f'<a:gradFill><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:path path="circle"><a:fillToRect l="50000" t="50000" r="50000" b="50000"/></a:path></a:gradFill>'),
        ("Rectangular", f'<a:gradFill><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:path path="rect"><a:fillToRect l="100000" t="100000"/></a:path></a:gradFill>'),
        ("Along path", f'<a:gradFill><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:path path="shape"><a:fillToRect/></a:path></a:gradFill>'),
    ]
    shapes = [placed(10 + i, name, f'{RECT}{fill}', at=(i % 8, 0)) for i, (name, fill) in enumerate(gradients)]
    for i, value in enumerate(enum_values("ST_PresetPatternVal")):
        fill = f'<a:pattFill prst="{value}"><a:fgClr>{RED}</a:fgClr><a:bgClr>{BLUE}</a:bgClr></a:pattFill>'
        shapes.append(placed(100 + i, value, f'{RECT}{fill}', at=(i % 8, 1 + i // 8)))
    outline = (f'{RECT}<a:noFill/><a:ln w="38100"><a:gradFill><a:gsLst>{stops(RED, BLUE)}</a:gsLst><a:lin ang="0" scaled="0"/></a:gradFill></a:ln>')
    shapes.append(placed(200, "Gradient line", outline, at=(0, 9)))
    hatched = (f'{RECT}<a:noFill/><a:ln w="38100"><a:pattFill prst="wdUpDiag"><a:fgClr>{RED}</a:fgClr><a:bgClr>{BLUE}</a:bgClr></a:pattFill></a:ln>')
    shapes.append(placed(201, "Pattern line", hatched, at=(1, 9)))
    return replace(deck(), "ppt/slides/slide1.xml", slide_of(*shapes))


SCENE = '<a:camera prst="perspectiveLeft" fov="2700000"><a:rot lat="0" lon="1200000" rev="0"/></a:camera><a:lightRig rig="threePt" dir="t"/>'
SP3D = '<a:bevelT w="63500" h="25400" prst="circle"/><a:contourClr><a:srgbClr val="00FF00"/></a:contourClr>'


def warp_deck():
    """A body's warp with its adjusts, its scene and each of the `EG_Text3D` choice."""
    bodies = [
        ("Warp only", '<a:bodyPr><a:prstTxWarp prst="textArchUp"><a:avLst><a:gd name="adj" fmla="val 10800000"/></a:avLst></a:prstTxWarp></a:bodyPr>'),
        ("Warp no list", '<a:bodyPr><a:prstTxWarp prst="textPlain"/></a:bodyPr>'),
        ("Warp empty list", '<a:bodyPr><a:prstTxWarp prst="textNoShape"><a:avLst/></a:prstTxWarp><a:normAutofit/></a:bodyPr>'),
        ("Scene and extrusion", f'<a:bodyPr><a:spAutoFit/><a:scene3d>{SCENE}</a:scene3d><a:sp3d z="12700" extrusionH="25400" contourW="1" prstMaterial="clear">{SP3D}</a:sp3d></a:bodyPr>'),
        ("Scene and flat", f'<a:bodyPr><a:scene3d>{SCENE}</a:scene3d><a:flatTx z="50800"/></a:bodyPr>'),
        ("Flat bare", '<a:bodyPr><a:flatTx/></a:bodyPr>'),
        ("Everything", '<a:bodyPr rot="60000" wrap="none"><a:prstTxWarp prst="textWave1"><a:avLst><a:gd name="adj1" fmla="val 12500"/><a:gd name="adj2" fmla="val 0"/></a:avLst></a:prstTxWarp>'
                       f'<a:noAutofit/><a:scene3d>{SCENE}</a:scene3d><a:sp3d/></a:bodyPr>'),
    ]
    shapes = [placed(10 + i, name, RECT, text_body(name, body), at=(i % 8, 0)) for i, (name, body) in enumerate(bodies)]
    return replace(deck(), "ppt/slides/slide1.xml", slide_of(*shapes))


def extension_deck():
    """Every `extLst` slot in 3D and a sketch, all reported; every path fill mode; and hidden
    properties in the older `a14` namespace, as `fixtures/real/sdk-animation.pptx` spells it."""
    def path(mode):
        return (f'<a:custGeom><a:pathLst><a:path w="100" h="100" fill="{mode}"><a:moveTo><a:pt x="0" y="0"/></a:moveTo>'
                '<a:lnTo><a:pt x="100" y="100"/></a:lnTo><a:close/></a:path></a:pathLst></a:custGeom>')
    shapes = [placed(10 + i, f"Path {mode}", path(mode), at=(i % 8, 0))
              for i, mode in enumerate(enum_values("ST_PathFillMode"))]
    shapes.append(placed(30, "Scene extension", (
        f'{RECT}<a:scene3d>{SCENE}<a:backdrop><a:anchor x="1" y="2" z="3"/><a:norm dx="4" dy="5" dz="6"/><a:up dx="7" dy="8" dz="9"/>'
        f'{FOREIGN_EXT}</a:backdrop>{FOREIGN_EXT}</a:scene3d><a:sp3d>{FOREIGN_EXT}</a:sp3d>'), at=(0, 1)))
    shapes.append(placed(31, "Sketch extension", (
        f'{RECT}<a:ln><a:solidFill>{RED}</a:solidFill><a:extLst><a:ext uri="{{C807C97D-BFC1-408E-A445-0C87EB9F89A2}}">'
        f'<ask:lineSketchStyleProps xmlns:ask="{ASK}" sd="7">{RECT}<ask:type><ask:lineSketchFreehand/></ask:type>'
        f'<ask:extLst><a:ext uri="{{7A9C1D30-5B1E-4F7A-8C2D-0E1F2A3B4C5D}}"><f:future xmlns:f="urn:example:future"/></a:ext></ask:extLst>'
        '</ask:lineSketchStyleProps></a:ext></a:extLst></a:ln>'), at=(1, 1)))
    shapes.append(placed(32, "Older namespace", (
        f'{RECT}<a:noFill/><a:extLst><a:ext uri="{{909E8E84-426E-40DD-AFC4-6F175D3DCCD1}}"><a14:hiddenFill xmlns:a14="{A14_OLD}"><a:solidFill>{RED}</a:solidFill></a14:hiddenFill></a:ext>'
        f'<a:ext uri="{{91240B29-F687-4F45-9708-019B960494DF}}"><a14:hiddenLine xmlns:a14="{A14_OLD}" w="9525"><a:solidFill>{BLUE}</a:solidFill></a14:hiddenLine></a:ext></a:extLst>'),
        at=(2, 1)))
    node_extension = '<p:extLst><p:ext uri="{7A9C1D30-5B1E-4F7A-8C2D-0E1F2A3B4C5D}"><f:future xmlns:f="urn:example:future"/></p:ext></p:extLst>'
    shapes.append(placed(33, "Node extension", RECT, f"{text_body('Node extension')}{node_extension}", at=(3, 1)))
    return replace(deck(), "ppt/slides/slide1.xml", slide_of(*shapes))


def definition(tag, body="", lst="", style=""):
    return f'<a:{tag}><a:spPr/>{body}{lst}{style}</a:{tag}>'


def run_default(size):
    return f'<a:lstStyle><a:lvl1pPr marL="{size}"><a:defRPr sz="{size}"/></a:lvl1pPr></a:lstStyle>'


def style_of(font="minor", color="lt1"):
    return ('<p:style><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="2"><a:schemeClr val="accent2"/></a:fillRef>'
            f'<a:effectRef idx="0"><a:schemeClr val="accent3"/></a:effectRef><a:fontRef idx="{font}"><a:schemeClr val="{color}"/></a:fontRef></p:style>')


def defaults_deck():
    """A theme's `txDef`, `lnDef` and `spDef`, each answering what the one before it does not,
    and shapes that hold a `p:style`, including the kinds on which `fontRef` is not used."""
    defaults = ('<a:objectDefaults>'
                + definition("spDef", '<a:bodyPr rot="60000" numCol="5" anchor="b"/>', run_default(2500),
                             style_of().replace("p:style", "a:style"))
                + definition("lnDef", '<a:bodyPr numCol="3"/>', run_default(2300))
                + definition("txDef", '<a:bodyPr anchor="ctr"/>', run_default(2100))
                + '</a:objectDefaults>')
    if "<a:objectDefaults/>" in THEME:
        theme = THEME.replace("<a:objectDefaults/>", defaults)
    else:
        theme = THEME.replace("</a:themeElements>", f"</a:themeElements>{defaults}")
    styled = placed(10, "Styled", RECT, text_body("Styled"), style_of("major", "lt1"), at=(0, 0))
    none_font = placed(11, "No font", RECT, text_body("No font"), style_of("none", "dk1"), at=(1, 0))
    bare = placed(12, "No style", RECT, text_body("No style"), at=(2, 0))
    connector = ('<p:cxnSp><p:nvCxnSpPr><p:cNvPr id="13" name="Connector"/><p:cNvCxnSpPr><a:stCxn id="10" idx="1"/><a:endCxn id="12" idx="3"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr>'
                 '<p:spPr><a:xfrm><a:off x="150000" y="900000"/><a:ext cx="1400000" cy="0"/></a:xfrm><a:prstGeom prst="straightConnector1"><a:avLst/></a:prstGeom></p:spPr>'
                 f'{style_of()}</p:cxnSp>')
    entries = replace(deck(), "ppt/theme/theme1.xml", theme)
    return replace(entries, "ppt/slides/slide1.xml", slide_of(styled, none_font, bare, connector))


def hostile():
    def with_props(props):
        return zipped(replace(deck(), "ppt/slides/slide1.xml", slide_of(placed(2, "s", props))))

    def with_body(body):
        return zipped(replace(deck(), "ppt/slides/slide1.xml", slide_of(placed(2, "s", RECT, f"<p:txBody>{body}<a:p/></p:txBody>"))))

    guide = '<a:gd name="a" fmla="val 1"/>'
    yield "hostile/text-limit-warp-guides.bin", with_body(
        f'<a:bodyPr><a:prstTxWarp prst="textPlain"><a:avLst>{guide * 4097}</a:avLst></a:prstTxWarp></a:bodyPr>'), "open"
    nested = '<a:fillOverlay blend="over"><a:blipFill><a:blip>' * 40 + '</a:blip></a:blipFill></a:fillOverlay>' * 40
    transforms = '<a:lumMod val="1"/>' * 200
    yield "hostile/text-bad-3d.bin", with_body(
        '<a:bodyPr><a:prstTxWarp prst="notAWarp"><a:avLst><a:gd/></a:avLst><a:foreign/></a:prstTxWarp><a:prstTxWarp prst="textPlain"/>'
        '<a:scene3d><a:camera prst="z" fov="-1"/><a:lightRig rig="z" dir="z"/><a:foreign/></a:scene3d><a:scene3d/>'
        f'<a:sp3d z="1e99" extrusionH="-1" prstMaterial="z"><a:bevelT w="-1"/><a:extrusionClr><a:srgbClr val="000000">{transforms}</a:srgbClr></a:extrusionClr></a:sp3d>'
        '<a:flatTx z="not-a-number" other="1"><a:foreign/></a:flatTx></a:bodyPr>'), "open"
    yield "hostile/text-limit-3d-depth.bin", with_body(
        f'<a:bodyPr><a:sp3d><a:extrusionClr><a:srgbClr val="000000">{transforms * 6}</a:srgbClr></a:extrusionClr></a:sp3d><a:foreign>{nested}</a:foreign></a:bodyPr>'), "open"
    marks = '<a14:foregroundMark x1="1" y1="2" x2="3" y2="4"/>' * 4097
    a14 = "http://schemas.microsoft.com/office/drawing/2010/main"
    yield "hostile/drawing-limit-picture-effects.bin", with_props(
        f'{RECT}<a:blipFill><a:blip r:embed="rId1"><a:extLst><a:ext uri="{{BEBA8EAE-BF5A-486C-A8C5-ECC9F3942E4B}}"><a14:imgProps xmlns:a14="{a14}">'
        f'<a14:imgLayer r:embed="rId1"><a14:imgEffect><a14:backgroundRemoval t="1" b="2" l="3" r="4">{marks}</a14:backgroundRemoval></a14:imgEffect></a14:imgLayer>'
        '</a14:imgProps></a:ext></a:extLst></a:blip></a:blipFill>'), "open"
    bad_style = ('<p:style><a:lnRef idx="x"><a:srgbClr val="nope"/></a:lnRef><a:lnRef idx="1"/><a:fillRef/><a:effectRef idx="-1"/>'
                 '<a:fontRef idx="wat"><a:foreign/></a:fontRef><a:foreign/></p:style><p:style/>')
    yield "hostile/shape-style-bad-values.bin", zipped(replace(deck(), "ppt/slides/slide1.xml", slide_of(placed(2, "s", RECT, style=bad_style)))), "open"
    tabs = '<a:tab pos="1"/>' * 1025
    bad_defaults = ('<a:objectDefaults><a:spDef><a:spPr/><a:bodyPr numCol="99" rot="x"><a:foreign/></a:bodyPr><a:bodyPr/><a:lstStyle><a:lvl1pPr marL="-1"/><a:foreign/></a:lstStyle></a:spDef>'
                    f'<a:lnDef><a:spPr/><a:bodyPr/><a:lstStyle><a:lvl1pPr><a:tabLst>{tabs}</a:tabLst></a:lvl1pPr></a:lstStyle></a:lnDef></a:objectDefaults>')
    theme = THEME.replace("<a:objectDefaults/>", bad_defaults) if "<a:objectDefaults/>" in THEME else THEME.replace("</a:themeElements>", f"</a:themeElements>{bad_defaults}")
    yield "hostile/theme-object-defaults-bad-values.bin", zipped(replace(deck(), "ppt/theme/theme1.xml", theme)), "open"


def fixtures():
    """(relative path, bytes, manifest expectation), in a fixed order."""
    yield "generated/shape-fills.pptx", zipped(fills_deck()), "open"
    yield "generated/text-warp-and-3d.pptx", zipped(warp_deck()), "open"
    yield "generated/drawing-extension-lists.pptx", zipped(extension_deck()), "open"
    yield "generated/text-object-defaults.pptx", zipped(defaults_deck()), "open"
    yield from hostile()
