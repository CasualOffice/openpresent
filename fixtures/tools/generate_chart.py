#!/usr/bin/env python3
"""Tier 1J fixtures: charts. **Synthetic:** written to the schemas, not by PowerPoint, so they
show we follow those and keep what is there, not what Office does with it. Real charts are in
fixtures/real (`T1J-01`).

Three kinds:

  chart-families    the 16 classic families as PowerPoint shapes them, with caches and axes
  chart-ex-layouts  the 8 cx: layouts behind `mc:AlternateContent`, with a picture fallback
  chart-every-row-* one deck per schema, walking the XSD itself: every attribute, every element,
                    and a part per value of the longest enumeration, so each enum value and
                    each choice branch appears. It shows coverage, and means nothing as a chart.
  hostile/chart-*   a deck past each bound in `ChartLimits`, and damage

Imported by generate.py, which writes them and lists them in the manifest. The expected values
are in crates/casual-slide-import/tests/chart.rs and crates/casual-slide-export/tests/chart.rs.
"""
import pathlib
import re
import xml.etree.ElementTree as ET

from generate import DECL, NS, TREE, deck, rels, replace, seeded, strict, zipped

ROOT = pathlib.Path(__file__).resolve().parents[2]
XS = "{http://www.w3.org/2001/XMLSchema}"
DML = "http://schemas.openxmlformats.org/drawingml/2006/main"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
C_NS = "http://schemas.openxmlformats.org/drawingml/2006/chart"
CX_NS = "http://schemas.microsoft.com/office/drawing/2014/chartex"
CS_NS = "http://schemas.microsoft.com/office/drawing/2012/chartStyle"
CDR_NS = "http://schemas.openxmlformats.org/drawingml/2006/chartDrawing"
MC_NS = "http://schemas.openxmlformats.org/markup-compatibility/2006"
A14_NS = "http://schemas.microsoft.com/office/drawing/2010/main"
CHART_TYPE = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml"
CHARTEX_TYPE = "application/vnd.ms-office.chartex+xml"
STYLE_TYPE = "application/vnd.ms-office.chartstyle+xml"
COLORS_TYPE = "application/vnd.ms-office.chartcolorstyle+xml"
SHAPES_TYPE = "application/vnd.openxmlformats-officedocument.drawingml.chartshapes+xml"
SHEET_TYPE = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
CHART_REL = f"{REL}/chart"
CHARTEX_REL = "http://schemas.microsoft.com/office/2014/relationships/chartEx"
STYLE_REL = "http://schemas.microsoft.com/office/2011/relationships/chartStyle"
COLORS_REL = "http://schemas.microsoft.com/office/2011/relationships/chartColorStyle"
SHAPES_REL = f"{REL}/chartUserShapes"
WORKBOOK_REL = f"{REL}/package"
# Not a workbook, and not a zip. Nothing may ever try to read it (SEC-P09).
WORKBOOK = b"PK\x03\x04 this is not a workbook, and nothing may open it"

SCHEMAS = {
    "c": ("schemas/ooxml/dml-chart.xsd", C_NS),
    "cdr": ("schemas/ooxml/dml-chartDrawing.xsd", CDR_NS),
    "c14": ("schemas/ms-extensions/ms-odrawxml-10.xsd", "http://schemas.microsoft.com/office/drawing/2007/8/2/chart"),
    "cdr14": ("schemas/ms-extensions/ms-odrawxml-08.xsd", "http://schemas.microsoft.com/office/drawing/2010/chartDrawing"),
    "c15": ("schemas/ms-extensions/ms-odrawxml-13.xsd", "http://schemas.microsoft.com/office/drawing/2012/chart"),
    "cs": ("schemas/ms-extensions/ms-odrawxml-15.xsd", CS_NS),
    "c16": ("schemas/ms-extensions/ms-odrawxml-20.xsd", "http://schemas.microsoft.com/office/drawing/2014/chart"),
    "c16ac": ("schemas/ms-extensions/ms-odrawxml-21.xsd", "http://schemas.microsoft.com/office/drawing/2014/chart/ac"),
    "cx": ("schemas/ms-extensions/ms-odrawxml-22.xsd", CX_NS),
    "c16r3": ("schemas/ms-extensions/ms-odrawxml-31.xsd", "http://schemas.microsoft.com/office/drawing/2017/03/chart"),
    "c16r2": ("schemas/ms-extensions/ms-odrawxml-42.xsd", "http://schemas.microsoft.com/office/drawing/2015/06/chart"),
}
EXT_PREFIXES = ("c14", "c15", "c16", "c16ac", "c16r2", "c16r3")
ALL_NS = (f'xmlns:a="{DML}" xmlns:r="{REL}" xmlns:mc="{MC_NS}" '
          + " ".join(f'xmlns:{p}="{SCHEMAS[p][1]}"' for p in SCHEMAS))
IGNORABLE = f'mc:Ignorable="{" ".join(EXT_PREFIXES + ("cdr14",))}"'
COLORS = ['scrgbClr r="1" g="2" b="3"', 'srgbClr val="4472C4"', 'hslClr hue="1" sat="2" lum="3"', 'sysClr val="window"',
          'schemeClr val="accent1"', 'prstClr val="black"']
FOREIGN = {
    "CT_ShapeProperties": '<a:solidFill><a:srgbClr val="4472C4"/></a:solidFill><a:ln w="9525"><a:noFill/></a:ln>',
    "CT_TextBody": '<a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>x</a:t></a:r></a:p>',
    "CT_ShapeStyle": ('<a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef>'
                      '<a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="accent1"/></a:fontRef>'),
    "CT_BlipFillProperties": "<a:blip/>",
    "CT_SolidColorFillProperties": '<a:srgbClr val="4472C4"/>',
}
FOREIGN_ATTRS = {
    "CT_NonVisualDrawingProps": 'id="2" name="x"',
    "CT_PositiveSize2D": 'cx="1000" cy="1000"',
    "CT_ColorMapping": ('bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" '
                        'accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"'),
}
RIDS = {"externalData": "rId1", "userShapes": "rId2", "legacyDrawingHF": "rId5"}


class Xsd:
    def __init__(self, prefix, path, uri):
        self.prefix, self.uri = prefix, uri
        self.ns = {}
        for _, (pfx, u) in ET.iterparse(ROOT / path, events=("start-ns",)):
            self.ns[pfx] = u
        root = ET.parse(ROOT / path).getroot()
        self.attr_qualified = root.get("attributeFormDefault") == "qualified"
        self.complex = {n.get("name"): n for n in root if n.tag == XS + "complexType"}
        self.simple = {n.get("name"): n for n in root if n.tag == XS + "simpleType"}
        self.groups = {n.get("name"): n for n in root if n.tag == XS + "group"}
        self.elements = {n.get("name"): n for n in root if n.tag == XS + "element"}

    def q(self, text):
        pfx, _, local = text.strip().rpartition(":")  # MS-ODRAWXML spells one type with a space.
        return self.ns.get(pfx, ""), local


XSDS = {p: Xsd(p, *SCHEMAS[p]) for p in SCHEMAS}
BY_URI = {x.uri: x for x in XSDS.values()}


def many(node):
    m = node.get("maxOccurs", "1")
    return m == "unbounded" or int(m) > 1


def builtin(local, variant):
    if local == "boolean":
        return "1" if variant % 2 else "0"
    if local == "double":
        return "1.5"
    if local in ("unsignedInt", "int", "integer", "byte", "unsignedByte", "unsignedShort"):
        return "1"
    if local == "language":
        return "en-US"
    if local == "base64Binary":
        return "AAAA"
    return "x"


def simple_value(x, uri, local, variant):
    if uri == "http://www.w3.org/2001/XMLSchema":
        return builtin(local, variant)
    other = BY_URI.get(uri)
    if other is None:
        return {"ST_Lang": "en-US", "ST_PositiveUniversalMeasure": "1in", "ST_BlackWhiteMode": "auto",
                "ST_StyleMatrixColumnIndex": "1", "ST_FontCollectionIndex": "minor"}.get(local, "x")
    node = other.simple.get(local)
    if node is None:
        return "x"
    return simple_node_value(other, node, variant)


def simple_node_value(x, node, variant):
    restriction = node.find(XS + "restriction")
    if restriction is not None:
        values = [e.get("value") for e in restriction.findall(XS + "enumeration")]
        if values:
            at = NEXT_VALUE.get(id(node), variant)
            NEXT_VALUE[id(node)] = at + 1
            return values[at % len(values)]
        bounds = {c.tag.replace(XS, ""): c.get("value") for c in restriction}
        base = restriction.get("base")
        if "pattern" in bounds:
            # Every percent pattern in these schemas takes a number from its numeric sibling.
            return None
        if "minInclusive" in bounds:
            v = bounds["minInclusive"]
            return v if "." in v or base.endswith("double") else v
        if "minExclusive" in bounds:
            return "1"
        return simple_value(x, *x.q(base), variant)
    lst = node.find(XS + "list")
    if lst is not None:
        return simple_value(x, *x.q(lst.get("itemType")), variant)
    union = node.find(XS + "union")
    members = [("n", m) for m in (union.get("memberTypes") or "").split()] + [("i", n) for n in union.findall(XS + "simpleType")]
    chosen = []
    for kind, m in members:
        value = simple_value(x, *x.q(m), variant) if kind == "n" else simple_node_value(x, m, variant)
        if value is not None:
            chosen.append(value)
    if not chosen:
        return "1"
    # A percent spelling is the first member, numeric form beside it; show both by turns.
    has_percent = any(kind == "n" and x.q(m)[1] in x.simple and x.simple[x.q(m)[1]].find(XS + "restriction") is not None
                      and x.simple[x.q(m)[1]].find(XS + "restriction").find(XS + "pattern") is not None for kind, m in members)
    if has_percent and variant % 2:
        return chosen[0] + "%"
    return chosen[variant % len(chosen)]


# Where each enumeration has reached, for one deck's walk: a value at a branch only some
# variants take would otherwise be missed. Same order, same bytes, each run.
NEXT_VALUE = {}


class Instance:
    """One instance of a schema, walking its declarations."""

    def __init__(self, variant, root_exts=True):
        self.variant = variant
        self.stack = []
        self.idx = 0
        self.root_exts = root_exts
        self.shape_id = 100

    def attributes(self, x, tnode, element_local):
        out = []
        holder = tnode
        sc = tnode.find(XS + "simpleContent")
        if sc is not None:
            holder = sc.find(XS + "extension")
        for a in holder.findall(XS + "attribute"):
            if a.get("ref"):
                out.append(f'r:{a.get("ref").split(":")[-1]}="{RIDS.get(element_local, "rId1")}"')
                continue
            name = a.get("name")
            typ = a.get("type")
            if typ:
                value = simple_value(x, *x.q(typ), self.variant)
            else:
                inner = a.find(XS + "simpleType")
                value = simple_node_value(x, inner, self.variant) if inner is not None else "x"
            if value is None:
                value = "1"
            if name == "idx" and element_local == "pt":
                value = str(self.idx)
                self.idx += 1
            qualified = x.attr_qualified or (x.prefix, name) == ("cx", "autoUpdate")
            out.append(f'{x.prefix + ":" if qualified else ""}{name}="{value}"')
        return " ".join(out)

    def element(self, x, local, qtype, optional=False):
        """An element named `local` in x's namespace, of a type."""
        uri, tname = qtype
        tag = f"{x.prefix}:{local}"
        owner = BY_URI.get(uri)
        if uri == DML or uri == A14_NS:
            attrs = FOREIGN_ATTRS.get(tname, "")
            if tname == "CT_NonVisualDrawingProps":
                self.shape_id += 1
                attrs = f'id="{self.shape_id}" name="x"'
            return f'<{tag}{" " + attrs if attrs else ""}>{FOREIGN.get(tname, "")}</{tag}>'
        if owner is not None and tname in owner.complex:
            if self.stack.count((owner.prefix, tname)) >= 2:  # a group in a group, and no further
                return ""
            self.stack.append((owner.prefix, tname))
            out = self.complex(owner, tname, tag, local)
            self.stack.pop()
            return out
        return f"<{tag}>{simple_value(x, uri, tname, self.variant) or '1'}</{tag}>"

    def complex(self, x, tname, tag, element_local):
        node = x.complex[tname]
        attrs = self.attributes(x, node, element_local)
        if tname == "CT_NumVal" and self.variant == 0:
            attrs += ' c16r2:formatcode2="x"'
        sc = node.find(XS + "simpleContent")
        if sc is not None:
            base = x.q(sc.find(XS + "extension").get("base"))
            text = simple_value(x, *base, self.variant) or "x"
            return f'<{tag} {attrs}>{text}</{tag}>'
        body = "".join(self.particle(x, ch, tname) for ch in node if ch.tag != XS + "attribute")
        return f'<{tag}{" " + attrs if attrs else ""}>{body}</{tag}>'

    def particle(self, x, node, owner):
        tag = node.tag.replace(XS, "")
        if tag == "sequence":
            return "".join(self.particle(x, ch, owner) for ch in node)
        if tag == "choice":
            branches = list(node)
            if many(node):
                return "".join(self.particle(x, ch, owner) for ch in branches)
            for i in range(len(branches)):
                out = self.particle(x, branches[(self.variant + i) % len(branches)], owner)
                if out:
                    return out
            return ""
        if tag == "group":
            uri, name = x.q(node.get("ref"))
            if uri == DML:
                return (f'<a:{COLORS[self.variant % len(COLORS)]}/>' if name == "EG_ColorChoice"
                        else '<a:lumMod val="50000"/>')
            other = BY_URI[uri]
            if name == "EG_ObjectChoices" and owner in ("CT_RelSizeAnchor", "CT_AbsSizeAnchor") and self.variant == 0:
                return content_part_alternate()  # This anchor's one object.
            return "".join(self.particle(other, ch, owner) for ch in other.groups[name])
        if tag == "element":
            # What the schema requires more than one of, such as `axId` and a stock chart's `ser`.
            times = max(int(node.get("minOccurs", "1")), 1)
            return "".join(self.one_element(x, node, owner) for _ in range(times))
        return ""

    def one_element(self, x, node, owner):
        if True:
            ref = node.get("ref")
            if ref:
                uri, local = x.q(ref)
                if uri == DML:
                    return f'<a:{local}><a:graphicData uri="x"/></a:{local}>'
                other = BY_URI[uri]
                return self.element(other, local, other.q(other.elements[local].get("type")))
            local = node.get("name")
            if local == "extLst":
                return self.extension_list(x, owner)
            if owner == "CT_ChartSpace" and local == "style" and x.prefix == "c" and self.variant:
                return style_alternate()
            return self.element(x, local, x.q(node.get("type")))

    def extension_list(self, x, owner):
        """Extension items at the root alone: a list everywhere would hold them all, everywhere."""
        if x.prefix == "cx" and owner == "CT_ChartSpace" and self.root_exts:
            uri = "{C3380CC4-5D6E-409C-BE32-E72D297353CC}"
            return f'<cx:extLst><cx:ext uri="{uri}"><c16:uniqueId val="{{00000000-0000-4000-8000-000000000000}}"/></cx:ext></cx:extLst>'
        if owner != "CT_ChartSpace" or x.prefix != "c" or not self.root_exts:
            return f"<{x.prefix}:extLst/>"
        items = []
        for pfx in EXT_PREFIXES:
            ext = XSDS[pfx]
            for local, node in ext.elements.items():
                uri = f"{{{hash_name(pfx, local)}}}"
                items.append(f'<c:ext uri="{uri}">{self.element(ext, local, ext.q(node.get("type")))}</c:ext>')
        # The schema has `uniqueId` only in a filter's exception: real decks write it here.
        items.append('<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}"><c16:uniqueId val="{00000000-0000-4000-8000-000000000000}"/></c:ext>')
        return f'<c:extLst>{"".join(items)}</c:extLst>'


def hash_name(pfx, local):
    """A GUID-shaped uri, the same each run. Nothing reads it, as real decks spell one many ways."""
    n = sum(ord(ch) * (i + 1) for i, ch in enumerate(pfx + local))
    return f"{n:08X}-0000-4000-8000-000000000000"


def style_alternate():
    return ('<mc:AlternateContent><mc:Choice Requires="c14"><c14:style val="102"/></mc:Choice>'
            '<mc:Fallback><c:style val="2"/></mc:Fallback></mc:AlternateContent>')


def content_part_alternate():
    fallback = ('<cdr:sp macro="" textlink=""><cdr:nvSpPr><cdr:cNvPr id="3" name="ink"/><cdr:cNvSpPr/></cdr:nvSpPr>'
                '<cdr:spPr/></cdr:sp>')
    part = Instance(0, False).element(XSDS["cdr14"], "contentPart", ("" + SCHEMAS["cdr14"][1], "CT_ContentPart"))
    return f'<mc:AlternateContent><mc:Choice Requires="cdr14">{part}</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>'


def every_row_root(prefix, local, variant):
    x = XSDS[prefix]
    node = x.elements[local]
    inst = Instance(variant)
    out = inst.element(x, local, x.q(node.get("type")))
    head = f"<{prefix}:{local}"
    return f"{DECL}{head} {ALL_NS} {IGNORABLE}" + out[len(head):]


def frame(sid, name, rid, extended=False):
    data = (f'<a:graphicData uri="{CX_NS}"><cx:chart xmlns:cx="{CX_NS}" r:id="{rid}"/></a:graphicData>' if extended else
            f'<a:graphicData uri="{C_NS}"><c:chart xmlns:c="{C_NS}" r:id="{rid}"/></a:graphicData>')
    shape = (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="{name}"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
             f'<p:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="3000000"/></p:xfrm><a:graphic>{data}</a:graphic></p:graphicFrame>')
    if not extended:
        return shape
    fallback = (f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>'
                '<a:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="3000000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:sp>')
    return (f'<mc:AlternateContent xmlns:mc="{MC_NS}"><mc:Choice xmlns:cx1="http://schemas.microsoft.com/office/drawing/2015/9/8/chartex" Requires="cx1">{shape}</mc:Choice>'
            f'<mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>')


def slide_of(*shapes):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


def chart_deck(charts, extra_parts=(), extra_types=(), workbook=True, extended=(), workbook_bytes=WORKBOOK):
    """A one-slide deck holding these chart parts, in order: `charts` is
    [(part file name, xml, chart's own extra rels)]. Each has a workbook first, rId1."""
    frames, slide_rels, parts, types = [], [], [], []
    for i, (name, xml, own) in enumerate(charts):
        is_ex = name in extended
        typ = CHARTEX_REL if is_ex else CHART_REL
        slide_rels.append((typ, f"../charts/{name}"))
        frames.append(frame(i + 2, f"Chart {i + 1}", f"rId{i + 2}", is_ex))
        parts.append((f"ppt/charts/{name}", xml))
        types.append((f"/ppt/charts/{name}", CHARTEX_TYPE if is_ex else CHART_TYPE))
        chart_rels = []
        if workbook:
            book = f"Workbook{i + 1}.xlsx"
            parts.append((f"ppt/embeddings/{book}", workbook_bytes))
            types.append((f"/ppt/embeddings/{book}", SHEET_TYPE))
            chart_rels.append((WORKBOOK_REL, f"../embeddings/{book}"))
        chart_rels += list(own)
        if chart_rels:
            parts.append((f"ppt/charts/_rels/{name}.rels", rels(*chart_rels)))
    entries = deck(slide_rels=slide_rels, parts=parts + list(extra_parts), types_extra=types + list(extra_types))
    return replace(entries, "ppt/slides/slide1.xml", slide_of(*frames))


# ---------------------------------------------------------------- the 16 families


def str_cache(ref, values):
    pts = "".join(f'<c:pt idx="{i}"><c:v>{v}</c:v></c:pt>' for i, v in enumerate(values))
    return f'<c:strRef><c:f>{ref}</c:f><c:strCache><c:ptCount val="{len(values)}"/>{pts}</c:strCache></c:strRef>'


def num_cache(ref, values):
    pts = "".join(f'<c:pt idx="{i}"><c:v>{v}</c:v></c:pt>' for i, v in enumerate(values))
    return f'<c:numRef><c:f>{ref}</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="{len(values)}"/>{pts}</c:numCache></c:numRef>'


CATS = ["North", "South", "East"]


def series(i, kind="cat"):
    col = "BCD"[i]
    head = (f'<c:ser><c:idx val="{i}"/><c:order val="{i}"/><c:tx>{str_cache(f"Sheet1!${col}$1", [f"Series {i + 1}"])}</c:tx>')
    vals = [str(10 * (i + 1) + j) for j in range(3)]
    uid = (f'<c:extLst><c:ext uri="{{C3380CC4-5D6E-409C-BE32-E72D297353CC}}" xmlns:c16="{SCHEMAS["c16"][1]}">'
           f'<c16:uniqueId val="{{0000000{i}-0000-4000-8000-000000000000}}"/></c:ext></c:extLst>')
    if kind == "xy":
        body = f'<c:xVal>{num_cache("Sheet1!$A$2:$A$4", ["1", "2", "3"])}</c:xVal><c:yVal>{num_cache(f"Sheet1!${col}$2:${col}$4", vals)}</c:yVal>'
        return f"{head}{body}<c:smooth val=\"0\"/>{uid}</c:ser>"
    if kind == "bubble":
        body = (f'<c:invertIfNegative val="0"/><c:xVal>{num_cache("Sheet1!$A$2:$A$4", ["1", "2", "3"])}</c:xVal>'
                f'<c:yVal>{num_cache(f"Sheet1!${col}$2:${col}$4", vals)}</c:yVal><c:bubbleSize>{num_cache("Sheet1!$E$2:$E$4", ["4", "5", "6"])}</c:bubbleSize>')
        return f"{head}{body}{uid}</c:ser>"
    body = f'<c:cat>{str_cache("Sheet1!$A$2:$A$4", CATS)}</c:cat><c:val>{num_cache(f"Sheet1!${col}$2:${col}$4", vals)}</c:val>'
    if kind == "bar":
        return f'{head}<c:invertIfNegative val="0"/>{body}{uid}</c:ser>'
    if kind == "pie":
        return f'{head}<c:explosion val="10"/>{body}{uid}</c:ser>'
    if kind == "line":
        return f'{head}<c:marker><c:symbol val="circle"/><c:size val="5"/></c:marker>{body}<c:smooth val="0"/>{uid}</c:ser>'
    return f"{head}{body}{uid}</c:ser>"


LABELS = ('<c:dLbls><c:showLegendKey val="0"/><c:showVal val="1"/><c:showCatName val="0"/><c:showSerName val="0"/>'
          '<c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls>')
SCALING = '<c:scaling><c:orientation val="minMax"/></c:scaling>'


def cat_ax(a, b):
    return (f'<c:catAx><c:axId val="{a}"/>{SCALING}<c:delete val="0"/><c:axPos val="b"/><c:numFmt formatCode="General" sourceLinked="1"/>'
            f'<c:majorTickMark val="out"/><c:minorTickMark val="none"/><c:tickLblPos val="nextTo"/><c:crossAx val="{b}"/><c:crosses val="autoZero"/>'
            '<c:auto val="1"/><c:lblAlgn val="ctr"/><c:lblOffset val="100"/><c:noMultiLvlLbl val="0"/></c:catAx>')


def val_ax(a, b, pos="l"):
    return (f'<c:valAx><c:axId val="{a}"/>{SCALING}<c:delete val="0"/><c:axPos val="{pos}"/><c:majorGridlines/><c:numFmt formatCode="General" sourceLinked="1"/>'
            f'<c:majorTickMark val="out"/><c:minorTickMark val="none"/><c:tickLblPos val="nextTo"/><c:crossAx val="{b}"/><c:crosses val="autoZero"/><c:crossBetween val="between"/></c:valAx>')


def ser_ax(a, b):
    return (f'<c:serAx><c:axId val="{a}"/>{SCALING}<c:delete val="0"/><c:axPos val="b"/><c:majorTickMark val="out"/><c:minorTickMark val="none"/>'
            f'<c:tickLblPos val="nextTo"/><c:crossAx val="{b}"/><c:crosses val="autoZero"/></c:serAx>')


AX2 = '<c:axId val="111"/><c:axId val="222"/>'
AX3 = AX2 + '<c:axId val="333"/>'
CAT_VAL = cat_ax(111, 222) + val_ax(222, 111)
CAT_VAL_SER = cat_ax(111, 222) + val_ax(222, 111) + ser_ax(333, 222)
VIEW3D = '<c:view3D><c:rotX val="15"/><c:rotY val="20"/><c:rAngAx val="1"/></c:view3D>'
TWO = lambda kind: series(0, kind) + series(1, kind)  # noqa: E731


def families():
    two_cat = TWO("cat")
    xy_axes = val_ax(111, 222, "b") + val_ax(222, 111)
    return [
        ("area", f'<c:areaChart><c:grouping val="standard"/><c:varyColors val="0"/>{two_cat}{AX2}</c:areaChart>', CAT_VAL, ""),
        ("area3D", f'<c:area3DChart><c:grouping val="standard"/><c:varyColors val="0"/>{two_cat}{AX3}</c:area3DChart>', CAT_VAL_SER, VIEW3D),
        ("line", f'<c:lineChart><c:grouping val="standard"/><c:varyColors val="0"/>{TWO("line")}{LABELS}<c:marker val="1"/>{AX2}</c:lineChart>', CAT_VAL, ""),
        ("line3D", f'<c:line3DChart><c:grouping val="standard"/><c:varyColors val="0"/>{two_cat}{AX3}</c:line3DChart>', CAT_VAL_SER, VIEW3D),
        ("stock", f'<c:stockChart>{series(0, "line")}{series(1, "line")}{series(2, "line")}<c:hiLowLines/>{AX2}</c:stockChart>', CAT_VAL, ""),
        ("radar", f'<c:radarChart><c:radarStyle val="marker"/><c:varyColors val="0"/>{two_cat}{AX2}</c:radarChart>', CAT_VAL, ""),
        ("scatter", f'<c:scatterChart><c:scatterStyle val="lineMarker"/><c:varyColors val="0"/>{TWO("xy")}{AX2}</c:scatterChart>', xy_axes, ""),
        ("pie", f'<c:pieChart><c:varyColors val="1"/>{series(0, "pie")}<c:firstSliceAng val="0"/></c:pieChart>', "", ""),
        ("pie3D", f'<c:pie3DChart><c:varyColors val="1"/>{series(0, "pie")}</c:pie3DChart>', "", VIEW3D),
        ("doughnut", f'<c:doughnutChart><c:varyColors val="1"/>{series(0, "pie")}<c:firstSliceAng val="0"/><c:holeSize val="75"/></c:doughnutChart>', "", ""),
        ("bar", f'<c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/>{TWO("bar")}{LABELS}<c:gapWidth val="150"/><c:overlap val="-27"/>{AX2}</c:barChart>', CAT_VAL, ""),
        ("bar3D", f'<c:bar3DChart><c:barDir val="bar"/><c:grouping val="clustered"/><c:varyColors val="0"/>{TWO("bar")}<c:gapWidth val="150"/><c:shape val="box"/>{AX2}</c:bar3DChart>', CAT_VAL, VIEW3D),
        ("ofPie", f'<c:ofPieChart><c:ofPieType val="pie"/><c:varyColors val="1"/>{series(0, "pie")}<c:gapWidth val="100"/><c:secondPieSize val="75"/></c:ofPieChart>', "", ""),
        ("surface", f'<c:surfaceChart><c:wireframe val="0"/>{two_cat}{AX3}</c:surfaceChart>', CAT_VAL_SER, ""),
        ("surface3D", f'<c:surface3DChart><c:wireframe val="0"/>{two_cat}{AX3}</c:surface3DChart>', CAT_VAL_SER, VIEW3D),
        ("bubble", f'<c:bubbleChart><c:varyColors val="0"/>{TWO("bubble")}<c:bubbleScale val="100"/>{AX2}</c:bubbleChart>', xy_axes, ""),
    ]


def chart_part(family, axes, view):
    title = ('<c:title><c:tx><c:rich><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>Sales</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/></c:title>')
    return (f'{DECL}<c:chartSpace xmlns:c="{C_NS}" xmlns:a="{DML}" xmlns:r="{REL}" xmlns:c14="{SCHEMAS["c14"][1]}" xmlns:mc="{MC_NS}">'
            f'<c:date1904 val="0"/><c:roundedCorners val="0"/>{style_alternate()}<c:chart>{title}<c:autoTitleDeleted val="0"/>{view}'
            f'<c:plotArea><c:layout/>{family}{axes}<c:spPr><a:noFill/></c:spPr></c:plotArea>'
            '<c:legend><c:legendPos val="r"/><c:overlay val="0"/></c:legend><c:plotVisOnly val="1"/><c:dispBlanksAs val="gap"/></c:chart>'
            '<c:spPr><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill><a:ln w="9525"><a:solidFill><a:srgbClr val="808080"/></a:solidFill></a:ln></c:spPr>'
            '<c:txPr><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr/></a:pPr><a:endParaRPr lang="en-US"/></a:p></c:txPr>'
            '<c:externalData r:id="rId1"><c:autoUpdate val="0"/></c:externalData></c:chartSpace>')


def families_deck():
    charts = [(f"chart{i + 1}.xml", chart_part(family, axes, view), [])
              for i, (_, family, axes, view) in enumerate(families())]
    return chart_deck(charts)


# ---------------------------------------------------------------- cx:


LAYOUTS = ["boxWhisker", "clusteredColumn", "funnel", "paretoLine", "regionMap", "sunburst", "treemap", "waterfall"]


def chart_ex_part(layout):
    return (f'{DECL}<cx:chartSpace xmlns:cx="{CX_NS}" xmlns:a="{DML}" xmlns:r="{REL}"><cx:chartData><cx:externalData r:id="rId1" cx:autoUpdate="0"/>'
            '<cx:data id="0"><cx:strDim type="cat"><cx:f>Sheet1!$A$2:$A$4</cx:f><cx:lvl ptCount="3"><cx:pt idx="0">North</cx:pt><cx:pt idx="1">South</cx:pt><cx:pt idx="2">East</cx:pt></cx:lvl></cx:strDim>'
            '<cx:numDim type="val"><cx:f>Sheet1!$B$2:$B$4</cx:f><cx:lvl ptCount="3" formatCode="General"><cx:pt idx="0">10</cx:pt><cx:pt idx="1">20</cx:pt><cx:pt idx="2">30</cx:pt></cx:lvl></cx:numDim></cx:data></cx:chartData>'
            f'<cx:chart><cx:title pos="t" align="ctr" overlay="0"/><cx:plotArea><cx:plotAreaRegion><cx:series layoutId="{layout}" uniqueId="{{00000000-0000-4000-8000-000000000000}}">'
            '<cx:tx><cx:txData><cx:f>Sheet1!$B$1</cx:f><cx:v>Sales</cx:v></cx:txData></cx:tx><cx:dataId val="0"/></cx:series></cx:plotAreaRegion>'
            '<cx:axis id="0"><cx:catScaling gapWidth="0.5"/><cx:tickLabels/></cx:axis><cx:axis id="1"><cx:valScaling/><cx:majorGridlines/><cx:tickLabels/></cx:axis></cx:plotArea>'
            '<cx:legend pos="t" align="ctr" overlay="0"/></cx:chart></cx:chartSpace>')


def chart_ex_deck():
    charts = [(f"chartEx{i + 1}.xml", chart_ex_part(layout), []) for i, layout in enumerate(LAYOUTS)]
    return chart_deck(charts, extended={c[0] for c in charts})


# ---------------------------------------------------------------- every row


def every_row_decks():
    """A deck per kind of root, with a part per value of the longest enumeration."""
    longest = {"c": 16, "cx": 13, "cs": 10}
    NEXT_VALUE.clear()
    c_charts = []
    extra_parts, extra_types = [], []
    for v in range(longest["c"]):
        shapes = f"drawing{v + 1}.xml"
        style = f"style{v + 1}.xml"
        colors = f"colors{v + 1}.xml"
        extra_parts += [(f"ppt/drawings/{shapes}", every_row_root("c", "userShapes", v)),
                        (f"ppt/charts/{style}", every_row_root("cs", "chartStyle", v)),
                        (f"ppt/charts/{colors}", every_row_root("cs", "colorStyle", v))]
        extra_types += [(f"/ppt/drawings/{shapes}", SHAPES_TYPE), (f"/ppt/charts/{style}", STYLE_TYPE), (f"/ppt/charts/{colors}", COLORS_TYPE)]
        c_charts.append((f"chart{v + 1}.xml", every_row_root("c", "chartSpace", v),
                         [(SHAPES_REL, f"../drawings/{shapes}"), (STYLE_REL, style), (COLORS_REL, colors)]))
    yield "generated/chart-every-row-classic.pptx", chart_deck(c_charts, extra_parts, extra_types)
    ex = [(f"chartEx{v + 1}.xml", every_row_root("cx", "chartSpace", v), []) for v in range(longest["cx"])]
    yield "generated/chart-every-row-ex.pptx", chart_deck(ex, extended={c[0] for c in ex})


# ---------------------------------------------------------------- hostile


def bare_chart(inner):
    return f'{DECL}<c:chartSpace xmlns:c="{C_NS}" xmlns:a="{DML}" xmlns:r="{REL}"><c:chart><c:plotArea>{inner}</c:plotArea></c:chart></c:chartSpace>'


def one_chart(xml, own=(), extra_parts=(), extra_types=()):
    return zipped(chart_deck([("chart1.xml", xml, list(own))], extra_parts, extra_types))


def hostile():
    # Each just past its default, so the part is set aside under that bound's code.
    ser = seeded('<c:ser><c:idx val="0"/><c:order val="0"/></c:ser>', 4_097)
    yield "hostile/chart-limit-series.bin", one_chart(bare_chart(f"<c:barChart>{ser}</c:barChart>")), "open"
    pts = seeded('<c:pt idx="0"><c:v>0</c:v></c:pt>', 1_048_577)
    yield "hostile/chart-limit-points.bin", one_chart(bare_chart(
        f'<c:barChart><c:ser><c:idx val="0"/><c:order val="0"/><c:val><c:numLit>{pts}</c:numLit></c:val></c:ser></c:barChart>')), "open"
    points = seeded('<c:dPt><c:idx val="0"/></c:dPt>', 262_145)
    yield "hostile/chart-limit-data-points.bin", one_chart(bare_chart(
        f'<c:barChart><c:ser><c:idx val="0"/><c:order val="0"/>{points}</c:ser></c:barChart>')), "open"
    labels = seeded('<c:dLbl><c:idx val="0"/></c:dLbl>', 262_145)
    yield "hostile/chart-limit-data-labels.bin", one_chart(bare_chart(
        f'<c:barChart><c:ser><c:idx val="0"/><c:order val="0"/><c:dLbls>{labels}</c:dLbls></c:ser></c:barChart>')), "open"
    levels = seeded('<c:lvl/>', 4_097)
    yield "hostile/chart-limit-levels.bin", one_chart(bare_chart(
        f'<c:barChart><c:ser><c:idx val="0"/><c:order val="0"/><c:cat><c:multiLvlStrRef><c:f>a</c:f><c:multiLvlStrCache>{levels}</c:multiLvlStrCache></c:multiLvlStrRef></c:cat></c:ser></c:barChart>')), "open"
    exts = seeded('<c:ext uri="x"/>', 262_145)
    yield "hostile/chart-limit-extensions.bin", one_chart(
        f'{DECL}<c:chartSpace xmlns:c="{C_NS}"><c:chart/><c:extLst>{exts}</c:extLst></c:chartSpace>'), "open"

    def with_drawing(xml):
        return one_chart(bare_chart(""), [(SHAPES_REL, "../drawings/drawing1.xml")],
                         [("ppt/drawings/drawing1.xml", xml)], [("/ppt/drawings/drawing1.xml", SHAPES_TYPE)])
    anchors = seeded('<cdr:relSizeAnchor/>', 65_537)
    yield "hostile/chart-limit-anchors.bin", with_drawing(f'{DECL}<c:userShapes xmlns:c="{C_NS}" xmlns:cdr="{CDR_NS}">{anchors}</c:userShapes>'), "open"
    deep = '<cdr:grpSp>' * 65 + '</cdr:grpSp>' * 65
    yield "hostile/chart-limit-group-depth.bin", with_drawing(
        f'{DECL}<c:userShapes xmlns:c="{C_NS}" xmlns:cdr="{CDR_NS}"><cdr:relSizeAnchor>{deep}</cdr:relSizeAnchor></c:userShapes>'), "open"

    # Everything reported, nothing refused. The marker must reach no finding.
    mark = "QzMarkerQz"
    bad = (f'{DECL}<c:chartSpace xmlns:c="{C_NS}" xmlns:a="{DML}" xmlns:r="{REL}" unknown="{mark}"><c:date1904 val="{mark}"/><c:date1904/><c:unknown/>'
           f'<c:chart unknown="{mark}"><c:autoTitleDeleted/><c:plotArea>'
           f'<c:barChart><c:barDir val="{mark}"/><c:ser><c:idx val="99999999999"/><c:idx val="0"/><c:order/><c:val><c:numRef><c:f><c:bad/></c:f>'
           f'<c:numCache><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="0"><c:v>2</c:v></c:pt><c:pt idx="9"><c:v>{mark}</c:v></c:pt><c:pt/></c:numCache></c:numRef></c:val></c:ser>'
           f'<c:ser><c:idx val="0"/></c:ser><c:ser><c:idx val="0"/></c:ser><c:gapWidth val="99999"/><c:overlap val="{mark}"/><c:axId val="9"/><c:axId val="{mark}"/><c:axId val="99999999999"/></c:barChart>'
           f'<c:catAx><c:axId val="-5"/><c:crossAx val="8"/><c:axPos val="{mark}"/></c:catAx><c:unknown/></c:plotArea></c:chart>'
           f'<c:externalData r:id="rIdNone"/><c:userShapes/></c:chartSpace>')
    yield "hostile/chart-bad-structure.bin", one_chart(bad), "open"
    # A chart-family relationship to a part that holds something else: set aside.
    yield "hostile/chart-wrong-root.bin", one_chart(f'{DECL}<c:notAChart xmlns:c="{C_NS}"/>'), "open"
    # A workbook that a reader would fail on: it is never read.
    yield "hostile/chart-workbook-corrupt.bin", zipped(chart_deck(
        [("chart1.xml", chart_part(*next((f, a, v) for _, f, a, v in families())), [])], workbook_bytes=b"\x00" * 4096)), "open"


def strict_chart(entries):
    """Strict by namespace mapping. **Synthetic:** as `generate.strict`."""
    pairs = [(C_NS, "http://purl.oclc.org/ooxml/drawingml/chart"), (CDR_NS, "http://purl.oclc.org/ooxml/drawingml/chartDrawing")]
    out = []
    for name, data in strict([(n, d) for n, d in entries if isinstance(d, str)]):
        for transitional, mapped in pairs:
            data = data.replace(transitional, mapped)
        # Strict has only the percent spelling of these.
        out.append((name, re.sub(r'<c:(lblOffset|gapWidth|overlap|holeSize|secondPieSize|bubbleScale) val="(-?\d+)"', r'<c:\1 val="\2%"', data)))
    binary = [(n, d) for n, d in entries if not isinstance(d, str)]
    return out + binary


def signed_axis_deck():
    """As `poi-bar-chart-mac`: Mac PowerPoint writes negative ids (`T1J-04`)."""
    family = (f'<c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/>{series(0, "bar")}'
              '<c:axId val="-1884094432"/><c:axId val="-2096236376"/></c:barChart>')
    axes = cat_ax(-1884094432, -2096236376) + val_ax(-2096236376, -1884094432)
    return chart_deck([("chart1.xml", chart_part(family, axes, ""), [])])


def fixtures():
    yield "generated/chart-families.pptx", zipped(families_deck()), "open"
    yield "generated/chart-strict.pptx", zipped(strict_chart(families_deck())), "open"
    yield "generated/chart-ex-layouts.pptx", zipped(chart_ex_deck()), "open"
    yield "generated/chart-signed-axis.pptx", zipped(signed_axis_deck()), "open"
    for name, entries in every_row_decks():
        yield name, zipped(entries), "open"
    yield from hostile()
