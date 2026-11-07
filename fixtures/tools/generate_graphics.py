#!/usr/bin/env python3
"""Tier 1K fixtures: SmartArt, equations and ink, and hostile ones. **Synthetic:** written from
the schemas by tools/spec-codegen/schema_parts.py, not by PowerPoint, so they show we follow
those and keep what is there, not what Office does with it. Real SmartArt is in fixtures/real.

`every_row` makes an instance of a root holding every element and attribute its schema
allows, and each value in turn. So a part can be nonsense as a diagram and valid to its schema:
`consistent_diagram` is the one that makes sense, with five parts that agree.

Imported by generate.py, which writes them and lists them in the manifest. The expected
values are in crates/casual-slide-export/tests/graphics.rs.
"""
import pathlib
import sys

from generate import CLRMAP, DECL, NS, TREE, deck, rels, replace, seeded, zipped  # noqa: F401

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "tools/spec-codegen"))
import schema_parts as sp  # noqa: E402

DML = sp.DML
A14 = "http://schemas.microsoft.com/office/drawing/2010/main"
P14 = "http://schemas.microsoft.com/office/powerpoint/2010/main"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
A = f'xmlns:a="{DML}"'
R = f'xmlns:r="{sp.REL}"'
REL_DATA = "diagramData"
REL_LAYOUT = "diagramLayout"
REL_STYLE = "diagramQuickStyle"
REL_COLORS = "diagramColors"
REL_DRAWING = "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing"
CT = {
    "data": "application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml",
    "layout": "application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml",
    "style": "application/vnd.openxmlformats-officedocument.drawingml.diagramStyle+xml",
    "colors": "application/vnd.openxmlformats-officedocument.drawingml.diagramColors+xml",
    "drawing": "application/vnd.ms-office.drawingml.diagramDrawing+xml",
    "ink": "application/inkml+xml",
}
GRAPHIC_DGM = "http://schemas.openxmlformats.org/drawingml/2006/diagram"
GUID = "{00000000-0000-4000-8000-00000000000%d}"
EXT_URI = "{00000000-0000-4000-8000-0000000000EE}"
DATA_EXT_URI = "http://schemas.microsoft.com/office/drawing/2008/diagram"

# How deep a type may recur, so that an instance holds each row without growing forever.
RECURSE = {"diagram": 2, "math": 1, "ink": 2}
# How many times one instance writes a type out in full, so that it stays small.
EXPANSIONS = 3
# Simple types of other schemas, and those whose patterns matter.
NAMED_VALUES = {
    "ST_Guid": GUID % 1, "ST_Percentage": "50%", "ST_String": "x", "ST_OnOff": "1", "ST_TwipsMeasure": "10",
    "ST_XAlign": "left", "ST_YAlign": "top", "ST_ShapeType": "rect", "ST_Point": "1,2", "ST_Points": "1,2 3,4",
    "ST_Ref": "#t1", "ST_StandardLengthUnits": "cm", "ST_StandardTimeUnits": "ms",
    "ST_LineEndType": "none", "ST_CalendarType": "gregorian",
}


class Maker:
    """One namespace-declaring pass. Each simple type's values, and each choice's branches,
    are taken in turn across every instance made by this object, so a few instances cover them."""

    def __init__(self, schemas, module):
        self.s, self.module = schemas, module
        self.turns = {}
        self.seen = set()
        self.ext_roots = [(sp.DGM14, "cNvPr"), (sp.DGM14, "recolorImg"), (sp.DGM1611, "autoBuNodeInfoLst"),
                          (sp.DGM1612, "spPr"), (sp.DGM1612, "lstStyle")]
        self.ids = 0
        self.expanded = {}

    def turn(self, key, options):
        at = self.turns.get(key, 0)
        self.turns[key] = at + 1
        return options[at % len(options)]

    def tokens(self, key, seen=()):
        """Everything a simple type can spell, as lexical forms."""
        st = self.s.simples.get(key)
        if key is None:
            return ["x"]
        if key[1] in NAMED_VALUES:
            return [NAMED_VALUES[key[1]]]
        if st is None:
            if key[0] == sp.XSD:
                return [{"boolean": "1", "int": "1", "integer": "1", "unsignedInt": "2", "long": "1", "double": "1.5",
                         "decimal": "1.5", "hexBinary": "0A", "anyURI": "urn:x", "ID": "i1"}.get(key[1], "x")]
            return ["x"]
        if key in seen:
            return []
        seen = (*seen, key)
        if st.values:
            return list(st.values)
        if st.union is not None:
            return [t for m in st.union for t in self.tokens(m, seen)]
        if st.list is not None:
            return self.tokens(st.list, seen)
        if st.min is not None:
            return [str(st.min)]
        if st.max is not None:
            return [str(st.max)]
        return self.tokens(st.base, seen) if st.base else ["x"]

    def attr_value(self, a):
        if a.rel:
            return ""  # As Office writes `r:blip`. A named one must exist in the part's rels.
        if a.name == "xml:space":
            return "preserve"  # XML 1.0 2.10: no other value parses in a validating reader.
        return self.turn(a.st, self.tokens(a.st) or ["x"])

    def attrs(self, t, required_only=False):
        out = []
        for a in t.attrs:
            if required_only and not a.required:
                continue
            value = self.attr_value(a)
            name = f"{self.s.prefix_of[t.ns]}:{a.name}" if a.qualified else a.name
            out.append(f'{name}="{value}"')
        return (" " + " ".join(out)) if out else ""

    def qname(self, ns, local):
        return f"{self.s.prefix_of.get(ns, 'a')}:{local}"

    def foreign(self, typ, q):
        local = typ[1]
        if local in ("CT_ShapeProperties", "CT_GroupShapeProperties"):
            geom = '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom>' if local == "CT_ShapeProperties" else ""
            return (f'<{q} bwMode="auto"><a:xfrm rot="1" flipH="0"><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></a:xfrm>{geom}'
                    f'<a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></{q}>')
        if local == "CT_TextBody":
            return f'<{q}><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>x</a:t></a:r></a:p></{q}>'
        if local == "CT_TextListStyle":
            return f'<{q}><a:lvl1pPr algn="ctr"/></{q}>'
        if local == "CT_ShapeStyle":
            return (f'<{q}><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef>'
                    f'<a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="lt1"/></a:fontRef></{q}>')
        if local == "CT_Transform2D":
            return f'<{q} rot="1"><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></{q}>'
        if local == "CT_NonVisualDrawingProps":
            self.ids += 1
            return f'<{q} id="{self.ids}" name="n" descr="d" hidden="0" title="t"/>'
        if local == "CT_NonVisualDrawingShapeProps":
            return f'<{q} txBox="1"/>'
        if local == "CT_NonVisualGroupDrawingShapeProps":
            return f"<{q}/>"
        if local == "CT_Scene3D":
            return f'<{q}><a:camera prst="orthographicFront"/><a:lightRig rig="flat" dir="t"/></{q}>'
        if local == "CT_Shape3D":
            return f'<{q} z="1"/>'
        if local == "CT_BackgroundFormatting":
            return f'<{q}><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill><a:effectLst/></{q}>'
        if local == "CT_WholeE2oFormatting":
            return f'<{q}><a:ln w="1"><a:noFill/></a:ln><a:effectLst/></{q}>'
        if local == "CT_OfficeArtExtensionList":
            root = self.turn("ext", self.ext_roots)
            typ = self.s.elements.get(root)
            body = self.root_body(root, typ)
            return f'<{q}><a:ext uri="{EXT_URI}">{body}</a:ext></{q}>'
        raise SystemExit(f"fixture for {typ}")

    def root_body(self, q, typ):
        name = self.qname(*q)
        if typ[0] == DML:
            return self.foreign(typ, name)
        return self.instance(typ, name, [], minimal=False)

    def group_children(self, t):
        out = []
        for g, _ in t.retained_groups:
            if g == (DML, "EG_ColorChoice"):
                out.append('<a:scrgbClr r="1" g="1" b="1"/><a:srgbClr val="0000FF"/><a:hslClr hue="1" sat="1" lum="1"/>'
                           '<a:sysClr val="window"/><a:schemeClr val="accent1"/><a:prstClr val="red"/>')
            elif g == (DML, "EG_Text3D"):
                out.append(self.turn("text3d", ['<a:sp3d z="1"/>', '<a:flatTx z="2"/>']))
            elif g == (DML, "EG_TextBullet"):
                out.append('<a:buChar char="x"/>')
        return "".join(out)

    def picks(self, t):
        """The child each non-repeating choice uses this time."""
        skip = set()
        for branches in t.choices:
            live = [b for b in branches if not next(c for c in t.children if c.local == b).in_items]
            if len(live) > 1:
                chosen = self.turn(("choice", t.ns, t.name, tuple(live)), live)
                skip |= {b for b in live if b != chosen}
        return skip

    def instance(self, key, name, stack, minimal=False):
        t = self.s.types[key]
        limit = RECURSE[self.module]
        if not minimal and (stack.count(key) >= limit or self.expanded.get(key, 0) >= EXPANSIONS):
            minimal = True
        if not minimal:
            self.expanded[key] = self.expanded.get(key, 0) + 1
        attrs = self.attrs(t, required_only=minimal)
        if t.text is not None:
            value = self.turn(t.text, self.tokens(t.text) or ["x"])
            return f"<{name}{attrs}>{value}</{name}>"
        if minimal:
            kids = "".join(self.child(c, stack, True) for c in t.children if c.required)
        else:
            stack = [*stack, key]
            skip = self.picks(t)
            parts = []
            for c in t.children:
                if c.local in skip:
                    continue
                parts.append(self.child(c, stack, False))
                if (c.repeats or c.in_items) and len(stack) < 2:
                    parts.append(self.child(c, stack, False))
            kids = "".join(parts) + self.group_children(t)
        return f"<{name}{attrs}>{kids}</{name}>" if kids else f"<{name}{attrs}/>"

    def child(self, c, stack, minimal):
        q = self.qname(c.ns, c.local)
        if c.type[0] == DML:
            return self.foreign(c.type, q)
        if c.type in self.s.types:
            return self.instance(c.type, q, stack, minimal)
        return f"<{q}>{self.turn(c.type, self.tokens(c.type) or ['x'])}</{q}>"

    def every_row(self, ns, local):
        typ = self.s.elements[(ns, local)]
        name = self.qname(ns, local)
        self.expanded = {}
        return self.instance(typ, name, [])


def namespaces(s):
    """Every prefix an instance may use, for its root."""
    out = [A, R, 'xmlns:xml_="urn:unused"'][:2]
    for sources in sp.SOURCES.values():
        for _, ns, prefix, _ in sources:
            out.append(f'xmlns:{prefix}="{ns}"')
    return " ".join(out)


def with_namespaces(s, xml):
    head, sep, rest = xml.partition(" ") if " " in xml.split(">", 1)[0] else (xml.split(">", 1)[0].rstrip("/"), "", "")
    if sep:
        return f"{DECL}{head} {namespaces(s)} {rest}"
    tail = xml[len(head):]
    return f"{DECL}{head} {namespaces(s)}{tail}"


def frame(sid, ids):
    return (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="Diagram {sid}"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
            f'<p:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="3000000"/></p:xfrm><a:graphic><a:graphicData uri="{GRAPHIC_DGM}">'
            f'<dgm:relIds xmlns:dgm="{GRAPHIC_DGM}" r:dm="rId{ids}" r:lo="rId{ids + 1}" r:qs="rId{ids + 2}" r:cs="rId{ids + 3}"/>'
            f'</a:graphicData></a:graphic></p:graphicFrame>')


def slide_of(*shapes):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


def diagram_deck(variants):
    """A deck whose slide 1 holds a frame per entry of (data, layout, style, colors, drawing)."""
    parts, types, links, frames = [], [], [], []
    for i, xml in enumerate(variants, 1):
        data, layout, style, colors, drawing = xml
        names = {"data": f"data{i}", "layout": f"layout{i}", "style": f"quickStyle{i}", "colors": f"colors{i}", "drawing": f"drawing{i}"}
        for role, body in zip(("data", "layout", "style", "colors", "drawing"), xml):
            if body is not None:
                parts.append((f"ppt/diagrams/{names[role]}.xml", body))
                types.append((f"/ppt/diagrams/{names[role]}.xml", CT[role]))
        frames.append(frame(100 + i, len(links) + 2))
        links += [(REL_DATA, f"../diagrams/{names['data']}.xml"), (REL_LAYOUT, f"../diagrams/{names['layout']}.xml"),
                  (REL_STYLE, f"../diagrams/{names['style']}.xml"), (REL_COLORS, f"../diagrams/{names['colors']}.xml"),
                  (REL_DRAWING, f"../diagrams/{names['drawing']}.xml")]
    entries = deck(parts=parts, types_extra=types, slide_rels=links)
    return replace(entries, "ppt/slides/slide1.xml", slide_of(*frames))


def every_row_diagram(s):
    """As many frames as it takes for every value of every type to have been used."""
    m = Maker(s, "diagram")
    roots = [("data", sp.DGM, "dataModel"), ("layout", sp.DGM, "layoutDef"), ("style", sp.DGM, "styleDef"),
             ("colors", sp.DGM, "colorsDef"), ("drawing", sp.DSP, "drawing")]
    variants = []
    rounds = 0
    while True:
        rounds += 1
        row = []
        for role, ns, local in roots:
            xml = m.every_row(ns, local)
            if role == "data":
                # Its own drawing, by the slide's relationship: rId<base + 4>. Relationship ids
                # follow the order frames were listed.
                base = 2 + 5 * len(variants)
                ext = (f'<dgm:extLst><a:ext uri="{DATA_EXT_URI}"><dsp:dataModelExt relId="rId{base + 4}" minVer="http://schemas.openxmlformats.org/drawingml/2006/diagram"/>'
                       f"</a:ext></dgm:extLst></dgm:dataModel>")
                head = xml.rsplit("<dgm:extLst>", 1)[0]
                xml = head + ext
            row.append(with_namespaces(s, xml))
        variants.append(tuple(row))
        if covered(m, s) or rounds >= 400:
            break
    # The standalone roots a deck does not link: headers and lists of them. Each is its part.
    extra = []
    for name in ("colorsDefHdr", "colorsDefHdrLst", "layoutDefHdr", "layoutDefHdrLst", "styleDefHdr", "styleDefHdrLst", "relIds"):
        extra.append((f"ppt/diagrams/{name}.xml", with_namespaces(s, m.every_row(sp.DGM, name))))
    for ns, name in ((sp.DGM14, "cNvPr"), (sp.DGM14, "recolorImg"), (sp.DGM1611, "autoBuNodeInfoLst"), (sp.DGM1612, "spPr"), (sp.DGM1612, "lstStyle")):
        extra.append((f"ppt/diagrams/ext-{name}-{sp.SOURCES and s.prefix_of[ns]}.xml", with_namespaces(s, m.root_body((ns, name), s.elements[(ns, name)]))))
    return diagram_deck(variants) + extra


def covered(m, s):
    """Whether each simple type in the module has been through all its values."""
    for key, st in s.simples.items():
        if st.module != m.module:
            continue
        options = m.tokens(key)
        used = m.turns.get(key)
        if used is not None and used < len(options):
            return False
    return True


# ---------------------------------------------------------------- one that makes sense

def consistent_parts():
    d = f'xmlns:dgm="{sp.DGM}" {A} {R}'
    p1, p2, doc, pres, sib, par, cx1 = (GUID % n for n in (1, 2, 3, 4, 5, 6, 7))
    data = (f'{DECL}<dgm:dataModel {d}><dgm:ptLst>'
            f'<dgm:pt modelId="{doc}" type="doc"><dgm:prSet loTypeId="urn:layout" loCatId="list" qsTypeId="urn:style" qsCatId="simple" csTypeId="urn:colors" csCatId="accent1"/><dgm:spPr/><dgm:t><a:bodyPr/><a:lstStyle/><a:p/></dgm:t></dgm:pt>'
            f'<dgm:pt modelId="{p1}"><dgm:prSet/><dgm:spPr/><dgm:t><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>First</a:t></a:r></a:p></dgm:t></dgm:pt>'
            f'<dgm:pt modelId="{par}" type="parTrans"><dgm:spPr/></dgm:pt><dgm:pt modelId="{sib}" type="sibTrans"><dgm:spPr/></dgm:pt>'
            f'<dgm:pt modelId="{pres}" type="pres"><dgm:prSet presAssocID="{p1}" presName="node" presStyleLbl="node1" presStyleIdx="0" presStyleCnt="1"/><dgm:spPr/></dgm:pt>'
            f'</dgm:ptLst><dgm:cxnLst><dgm:cxn modelId="{cx1}" srcId="{doc}" destId="{p1}" srcOrd="0" destOrd="0" parTransId="{par}" sibTransId="{sib}"/></dgm:cxnLst>'
            f'<dgm:bg/><dgm:whole/><dgm:extLst><a:ext uri="{DATA_EXT_URI}"><dsp:dataModelExt xmlns:dsp="{sp.DSP}" relId="rId6" minVer="http://schemas.openxmlformats.org/drawingml/2006/diagram"/></a:ext></dgm:extLst></dgm:dataModel>')
    layout = (f'{DECL}<dgm:layoutDef {d} uniqueId="urn:layout"><dgm:title val="Simple"/><dgm:desc val=""/><dgm:catLst><dgm:cat type="list" pri="400"/></dgm:catLst>'
              f'<dgm:layoutNode name="linear"><dgm:alg type="lin"/><dgm:shape r:blip=""><dgm:adjLst/></dgm:shape><dgm:forEach name="f" axis="ch" ptType="node"><dgm:layoutNode name="node"><dgm:alg type="tx"/><dgm:shape type="rect" r:blip=""><dgm:adjLst/></dgm:shape><dgm:presOf axis="desOrSelf" ptType="node"/></dgm:layoutNode></dgm:forEach></dgm:layoutNode></dgm:layoutDef>')
    label = '<a:lnRef idx="1"><a:scrgbClr r="0" g="0" b="0"/></a:lnRef><a:fillRef idx="1"><a:scrgbClr r="0" g="0" b="0"/></a:fillRef><a:effectRef idx="0"><a:scrgbClr r="0" g="0" b="0"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="lt1"/></a:fontRef>'
    style = (f'{DECL}<dgm:styleDef {d} uniqueId="urn:style"><dgm:title val=""/><dgm:desc val=""/><dgm:catLst><dgm:cat type="simple" pri="10100"/></dgm:catLst>'
             f'<dgm:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></dgm:scene3d><dgm:styleLbl name="node1"><dgm:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></dgm:scene3d><dgm:sp3d/><dgm:txPr/><dgm:style>{label}</dgm:style></dgm:styleLbl></dgm:styleDef>')
    colors = (f'{DECL}<dgm:colorsDef {d} uniqueId="urn:colors"><dgm:title val=""/><dgm:desc val=""/><dgm:catLst><dgm:cat type="accent1" pri="11200"/></dgm:catLst>'
              f'<dgm:styleLbl name="node1"><dgm:fillClrLst meth="repeat"><a:schemeClr val="accent1"/></dgm:fillClrLst><dgm:linClrLst meth="repeat"><a:schemeClr val="lt1"/></dgm:linClrLst><dgm:effectClrLst/><dgm:txLinClrLst/><dgm:txFillClrLst/><dgm:txEffectClrLst/></dgm:styleLbl></dgm:colorsDef>')
    drawing = (f'{DECL}<dsp:drawing xmlns:dgm="{sp.DGM}" xmlns:dsp="{sp.DSP}" {A}><dsp:spTree><dsp:nvGrpSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvGrpSpPr/></dsp:nvGrpSpPr><dsp:grpSpPr/>'
               f'<dsp:sp modelId="{pres}"><dsp:nvSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvSpPr/></dsp:nvSpPr><dsp:spPr><a:xfrm><a:off x="10" y="20"/><a:ext cx="300" cy="400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></dsp:spPr>'
               f'<dsp:style>{label}</dsp:style><dsp:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>First</a:t></a:r></a:p></dsp:txBody><dsp:txXfrm><a:off x="10" y="20"/><a:ext cx="300" cy="400"/></dsp:txXfrm></dsp:sp></dsp:spTree></dsp:drawing>')
    return data, layout, style, colors, drawing


# ---------------------------------------------------------------- equations

def every_math(s):
    m = Maker(s, "math")
    parts = []
    # A part per round until each value has been used: OMML stands alone in no presentation
    # part, so these exist for the row walk and for the part-level writer.
    body = None
    for i in range(1, 40):
        body = m.every_row(sp.MATH, "oMath")
        parts.append((f"ppt/math/oMath{i}.xml", with_namespaces(s, body)))
        parts.append((f"ppt/math/oMathPara{i}.xml", with_namespaces(s, m.every_row(sp.MATH, "oMathPara"))))
        parts.append((f"ppt/math/mathPr{i}.xml", with_namespaces(s, m.every_row(sp.MATH, "mathPr"))))
        if covered(m, s):
            break
    return parts, body


def math_namespace(xml):
    return xml.replace("<m:oMath>", f'<m:oMath xmlns:m="{sp.MATH}">', 1).replace("<m:oMathPara>", f'<m:oMathPara xmlns:m="{sp.MATH}">', 1)


def office_equation():
    """What Office writes: `a:rPr` in the runs, and in `m:ctrlPr`."""
    rpr = '<a:rPr lang="en-US" i="1"><a:latin typeface="Cambria Math"/></a:rPr>'
    return (f'<m:oMathPara xmlns:m="{sp.MATH}"><m:oMathParaPr><m:jc m:val="center"/></m:oMathParaPr><m:oMath>'
            f'<m:f><m:fPr><m:ctrlPr>{rpr}</m:ctrlPr></m:fPr><m:num><m:r>{rpr}<m:t>a</m:t></m:r></m:num><m:den><m:r>{rpr}<m:t>b</m:t></m:r></m:den></m:f>'
            f'<m:r><m:rPr><m:sty m:val="p"/></m:rPr>{rpr}<m:t xml:space="preserve"> + c</m:t></m:r></m:oMath></m:oMathPara>')


def math_deck(s):
    parts, body = every_math(s)
    full = math_namespace(body)
    inline = (f'<a:p><a:r><a:t>Before</a:t></a:r><mc:AlternateContent xmlns:mc="{MC}" xmlns:a14="{A14}"><mc:Choice Requires="a14"><a14:m>{office_equation()}</a14:m></mc:Choice>'
              f'<mc:Fallback><a:r><a:rPr lang="en-US"/><a:t>a/b + c</a:t></a:r></mc:Fallback></mc:AlternateContent><a:r><a:t>After</a:t></a:r></a:p>')
    every = (f'<a:p><mc:AlternateContent xmlns:mc="{MC}" xmlns:a14="{A14}"><mc:Choice Requires="a14"><a14:m>{full}</a14:m></mc:Choice>'
             f'<mc:Fallback><a:r><a:t>every</a:t></a:r></mc:Fallback></mc:AlternateContent></a:p>')
    sp_body = '<p:nvSpPr><p:cNvPr id="%d" name="%s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/>%s</p:txBody>'
    # PowerPoint's own form: the whole shape in the Choice, a picture-filled one in the Fallback.
    whole = (f'<mc:AlternateContent xmlns:mc="{MC}" xmlns:a14="{A14}"><mc:Choice Requires="a14"><p:sp>{sp_body % (12, "Whole", f"<a:p><a14:m>{office_equation()}</a14:m></a:p>")}</p:sp></mc:Choice>'
             f'<mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="12" name="Whole"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>'
             f'<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t> </a:t></a:r></a:p></p:txBody></p:sp></mc:Fallback></mc:AlternateContent>')
    slide = slide_of(f"<p:sp>{sp_body % (10, 'Inline', inline)}</p:sp>", f"<p:sp>{sp_body % (11, 'Every', every)}</p:sp>", whole)
    return replace(deck(parts=parts), "ppt/slides/slide1.xml", slide)


# ---------------------------------------------------------------- ink

def every_ink(s):
    m = Maker(s, "ink")
    parts = []
    for i in range(1, 60):
        parts.append((f"ppt/ink/ink{i}.xml", with_namespaces(s, m.every_row(sp.INKML, "ink"))))
        parts.append((f"ppt/ink/actions{i}.xml", with_namespaces(s, m.every_row(sp.IACT, "actions"))))
        parts.append((f"ppt/ink/context{i}.xml", with_namespaces(s, m.every_row(sp.MSINK, "context"))))
        if covered(m, s):
            break
    return parts


def real_form_ink():
    """As a pen writes it: one brush, one stroke."""
    ink = (f'{DECL}<inkml:ink xmlns:inkml="{sp.INKML}" xmlns:msink="{sp.MSINK}"><inkml:definitions><inkml:context xml:id="ctx0"><inkml:inkSource xml:id="inkSrc0"><inkml:traceFormat>'
           '<inkml:channel name="X" type="integer" max="32767" units="cm"/><inkml:channel name="Y" type="integer" max="32767" units="cm"/><inkml:channel name="F" type="integer" max="4096" units="dev"/>'
           '</inkml:traceFormat></inkml:inkSource><inkml:timestamp xml:id="ts0" timeString="2020-01-01T00:00:00.000"/></inkml:context>'
           '<inkml:brush xml:id="br0"><inkml:brushProperty name="width" value="0.05" units="cm"/><inkml:brushProperty name="height" value="0.05" units="cm"/>'
           '<inkml:brushProperty name="inkEffects" value="pencil"/><inkml:brushProperty name="anchorX" value="1"/><inkml:brushProperty name="anchorY" value="1"/><inkml:brushProperty name="scaleFactor" value="1"/></inkml:brush></inkml:definitions>'
           '<inkml:trace contextRef="#ctx0" brushRef="#br0">1 2 3, 4 5 6</inkml:trace>'
           '<inkml:annotationXML><msink:context type="inkDrawing" rotatedBoundingBox="1,2 3,4"/></inkml:annotationXML></inkml:ink>')
    effects = ["none", "rainbow", "galaxy", "gold", "silver", "lava", "ocean", "rosegold", "bronze"]
    more = "".join(f'<inkml:brushProperty name="inkEffects" value="{e}"/>' for e in effects)
    return ink.replace("</inkml:brush>", more + "</inkml:brush>", 1)


CUSTOM_XML = "customXml"
CONTENT_PART = (f'<mc:AlternateContent xmlns:mc="{MC}" xmlns:p14="{P14}"><mc:Choice Requires="p14"><p:contentPart bwMode="auto" r:id="rId2"><p:nvContentPartPr><p:cNvPr id="20" name="Ink 1"/><p:cNvContentPartPr/><p:nvPr/></p:nvContentPartPr><p:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></p:xfrm></p:contentPart></mc:Choice>'
                '<mc:Fallback><p:pic><p:nvPicPr><p:cNvPr id="20" name="Ink 1"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic></mc:Fallback></mc:AlternateContent>')


def ink_deck(s):
    parts = [("ppt/ink/ink0.xml", real_form_ink()), *every_ink(s)]
    types = [(f"/{n}", CT["ink"]) for n, _ in parts]
    entries = deck(parts=parts, types_extra=types, slide_rels=[(CUSTOM_XML, "../ink/ink0.xml")])
    return replace(entries, "ppt/slides/slide1.xml", slide_of(CONTENT_PART))


# ---------------------------------------------------------------- hostile

def one_diagram(data=None, layout=None, style=None, colors=None, drawing=None):
    d, lo, st, co, dr = consistent_parts()
    return zipped(diagram_deck([(data or d, layout or lo, style or st, colors or co, drawing or dr)]))


def hostile(s):
    dgm = f'xmlns:dgm="{sp.DGM}" {A} {R}'
    dsp = f'xmlns:dsp="{sp.DSP}" xmlns:dgm="{sp.DGM}" {A}'
    points = seeded('<dgm:pt modelId="1"/>', 65_537)
    yield "hostile/graphics-limit-points.bin", one_diagram(data=f"{DECL}<dgm:dataModel {dgm}><dgm:ptLst>{points}</dgm:ptLst></dgm:dataModel>"), "open"
    cxn = seeded('<dgm:cxn modelId="1" srcId="1" destId="1" srcOrd="0" destOrd="0"/>', 131_073)
    yield "hostile/graphics-limit-connections.bin", one_diagram(data=f'{DECL}<dgm:dataModel {dgm}><dgm:ptLst/><dgm:cxnLst>{cxn}</dgm:cxnLst></dgm:dataModel>'), "open"
    nodes = seeded('<dgm:layoutNode name="n"/>', 65_537)
    yield "hostile/graphics-limit-layout-nodes.bin", one_diagram(layout=f"{DECL}<dgm:layoutDef {dgm}><dgm:layoutNode>{nodes}</dgm:layoutNode></dgm:layoutDef>"), "open"
    tree = '<dsp:nvGrpSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvGrpSpPr/></dsp:nvGrpSpPr><dsp:grpSpPr/>'
    shapes = seeded('<dsp:sp modelId="1"/>', 65_537)
    yield "hostile/graphics-limit-drawing-shapes.bin", one_diagram(drawing=f"{DECL}<dsp:drawing {dsp}><dsp:spTree>{tree}{shapes}</dsp:spTree></dsp:drawing>"), "open"
    deep = 200
    yield "hostile/graphics-limit-depth.bin", one_diagram(layout=f'{DECL}<dgm:layoutDef {dgm}>{"<dgm:layoutNode>" * deep}{"</dgm:layoutNode>" * deep}</dgm:layoutDef>'), "open"
    # A definition that would take for ever if anything ran it: a loop of the largest count in
    # a loop, to the depth allowed. Nothing runs it, so it costs its size. SEC-C12.
    loops = 100
    bomb = '<dgm:forEach axis="ch" cnt="4294967295" ptType="all">' * loops + "</dgm:forEach>" * loops
    yield "hostile/graphics-layout-loop-bomb.bin", one_diagram(layout=f'{DECL}<dgm:layoutDef {dgm} uniqueId="urn:layout"><dgm:layoutNode>{bomb}</dgm:layoutNode></dgm:layoutDef>'), "open"
    # Damage, reported and nothing refused: a part of the wrong kind, parts that disagree. The
    # marker must reach no finding. One that is not XML is built in tests/graphics.rs, as
    # every "open" deck in the corpus is held to parse part by part.
    mark = "QzMarkerQz"
    yield "hostile/graphics-wrong-root.bin", one_diagram(style=f'{DECL}<dgm:notADefinition {dgm} a="{mark}"/>', colors=f"{DECL}<dgm:layoutDef {dgm}/>"), "open"
    bad = (f'{DECL}<dgm:dataModel {dgm}><dgm:ptLst><dgm:pt modelId="{mark}" type="{mark}" cxnId="{mark}" unknown="{mark}"><dgm:prSet presAssocID="{mark}" custScaleX="{mark}"/><dgm:prSet/><dgm:wat/></dgm:pt>'
           f'<dgm:pt modelId="7"/><dgm:pt modelId="7"/></dgm:ptLst><dgm:cxnLst><dgm:cxn modelId="8" srcId="99" destId="98"/><dgm:cxn/></dgm:cxnLst><dgm:extLst><a:ext uri="x"><dsp:dataModelExt xmlns:dsp="{sp.DSP}" relId="rId999"/></a:ext></dgm:extLst></dgm:dataModel>')
    yield "hostile/graphics-bad-data.bin", one_diagram(data=bad), "open"
    # Inline math over the depth bound, and an ink part with too many traces.
    frac = '<m:f><m:num>' * 80 + "</m:num></m:f>" * 80
    equation = f'<a:p><mc:AlternateContent xmlns:mc="{MC}" xmlns:a14="{A14}"><mc:Choice Requires="a14"><a14:m><m:oMath xmlns:m="{sp.MATH}">{frac}</m:oMath></a14:m></mc:Choice><mc:Fallback><a:r><a:t>x</a:t></a:r></mc:Fallback></mc:AlternateContent></a:p>'
    body = f'<p:sp><p:nvSpPr><p:cNvPr id="10" name="Deep"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/>{equation}</p:txBody></p:sp>'
    yield "hostile/graphics-limit-math-depth.bin", zipped(replace(deck(), "ppt/slides/slide1.xml", slide_of(body))), "open"
    traces = seeded('<inkml:trace>1 2</inkml:trace>', 262_145)
    ink = f'{DECL}<inkml:ink xmlns:inkml="{sp.INKML}">{traces}</inkml:ink>'
    entries = deck(parts=[("ppt/ink/ink1.xml", ink)], types_extra=[("/ppt/ink/ink1.xml", CT["ink"])], slide_rels=[(CUSTOM_XML, "../ink/ink1.xml")])
    yield "hostile/graphics-limit-traces.bin", zipped(replace(entries, "ppt/slides/slide1.xml", slide_of(CONTENT_PART))), "open"


def fixtures():
    s = sp.Schemas()
    yield "generated/graphics-diagram-every-row.pptx", zipped(every_row_diagram(s)), "open"
    yield "generated/graphics-diagram-consistent.pptx", zipped(diagram_deck([consistent_parts()])), "open"
    yield "generated/graphics-math.pptx", zipped(math_deck(s)), "open"
    yield "generated/graphics-ink.pptx", zipped(ink_deck(s)), "open"
    yield from hostile(s)
