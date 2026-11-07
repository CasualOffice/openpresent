"""Tier 1L fixtures: OLE objects, ActiveX controls, legacy VML and 3D models, kept as bytes.

The VML part and the 3D model's markup are written from the vendored schemas, every element,
attribute and enumerated value, so each row marked `r` has a deck holding it. Everything else
is what PowerPoint writes in shape: an `mc:AlternateContent` with the picture in its fallback.
"""
import pathlib
import sys
from xml.sax.saxutils import quoteattr

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/spec-audit"))
import audit  # noqa: E402

from generate import DECL, NS, TITLE, TREE, deck, rels, zipped  # noqa: E402

PNG = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                    "e221bc330000000049454e44ae426082")
# Not real objects. Nothing here may ever try to read one, and these say so if it does.
OLE_BYTES = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1 retained, never opened"
CONTROL_BYTES = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1 retained, never instantiated"
MODEL_BYTES = b"glTF\x02\x00\x00\x00 retained, never decoded"
MODEL3D_REL = "http://schemas.microsoft.com/office/2017/06/relationships/model3d"
ACTIVEX_BIN_REL = "http://schemas.microsoft.com/office/2006/relationships/activeXControlBinary"
AM3D = "http://schemas.microsoft.com/office/drawing/2017/model3d"
AM3D_ANIM = "http://schemas.microsoft.com/office/drawing/2018/animation/model3d"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
MC_NS = "http://schemas.openxmlformats.org/markup-compatibility/2006"
V = "urn:schemas-microsoft-com:vml"
O = "urn:schemas-microsoft-com:office:office"
PVML = "urn:schemas-microsoft-com:office:powerpoint"
PREFIX = {V: "v", O: "o", PVML: "pvml", AM3D: "am3d", AM3D_ANIM: "am3danim", A: "a", R: "r",
          "http://schemas.microsoft.com/office/drawing/2018/animation": "aanim",
          "http://schemas.microsoft.com/office/drawing/2010/main": "a14",
          "http://schemas.openxmlformats.org/officeDocument/2006/sharedTypes": "s",
          "urn:schemas-microsoft-com:office:excel": "x", "urn:schemas-microsoft-com:office:word": "w10"}
BUILTIN = {"boolean": "true", "dateTime": "2026-01-01T00:00:00Z", "base64Binary": "AA=="}
OLE_URI = "http://schemas.openxmlformats.org/presentationml/2006/ole"


class Writer:
    """One instance of everything reachable from an element, by the schema the audit reads."""

    def __init__(self, schema, rel_id):
        self.schema = schema
        self.rel_id = rel_id
        self.done = set()
        self.used = {}

    def name(self, q):
        ns, local = q
        if not ns:
            return local
        prefix = PREFIX.get(ns)
        if prefix is None:
            prefix = f"n{len(PREFIX)}"
            PREFIX[ns] = prefix
        self.used[prefix] = ns
        return f"{prefix}:{local}"

    def values(self, typ, seen=None):
        """Every value an enumeration, or a union of them, names."""
        seen = seen if seen is not None else set()
        s = self.schema.simples.get(typ)
        if not s or typ in seen:
            return []
        seen.add(typ)
        out = list(s["values"])
        for member in s["members"]:
            out += self.values(member, seen)
        return out

    def one_value(self, typ, q):
        if q[0] == R:
            return self.rel_id
        values = self.values(typ)
        if values:
            return values[0]
        return BUILTIN.get(typ[-1] if isinstance(typ, tuple) else typ, "1")

    def attributes(self, flat):
        """All of them with a first value, and what else each may say."""
        attrs, others = [], []
        for q, entries in sorted(flat["attrs"].items()):
            decl = entries[0][1]
            typ = decl["type"]
            if decl["ref"] and q in self.schema.attributes:
                typ = typ or self.schema.attributes[q]["type"]
            if q[0] == "http://www.w3.org/XML/1998/namespace":
                continue
            value = self.one_value(typ, q)
            attrs.append((self.name(q), value))
            others += [(self.name(q), v) for v in self.values(typ)[1:]]
        return attrs, others

    def element(self, q, typ):
        name = self.name(q)
        if typ in self.schema.simples:
            values = self.values(typ) or [self.one_value(typ, q)]
            return "".join(f"<{name}>{v}</{name}>" for v in values)
        flat = self.schema.flat(typ)
        attrs, others = self.attributes(flat)
        head = "".join(f" {k}={quoteattr(v)}" for k, v in attrs)
        out = []
        if typ in self.done:
            return f"<{name}{head}/>"
        self.done.add(typ)
        body = []
        for child, entries in flat["children"].items():
            decl = entries[0][1]
            ctype = decl["type"]
            if decl["ref"] or ctype is None:
                glob = self.schema.elements.get(child)
                ctype = glob["type"] if glob else ctype
            body.append(self.element(child, ctype))
        if flat["any"]:
            body.append('<fixture:any xmlns:fixture="urn:openpresent:fixture"/>')
        out.append(f"<{name}{head}>" + "".join(body) + f"</{name}>")
        out += [f"<{name} {k}={quoteattr(v)}/>" for k, v in others]
        return "".join(out)

    def declarations(self):
        return "".join(f' xmlns:{p}="{ns}"' for p, ns in sorted(self.used.items()))


def vml_part(schema):
    w = Writer(schema, "rId1")
    body = []
    for q, e in sorted(schema.elements.items()):
        if q[0] in (V, O, PVML):
            body.append(w.element(q, e["type"]))
    # Declared globally and used by no type: it is only ever found on a shape like this.
    body.append(f'<v:shape {w.name((O, "wrapcoords"))}="0,0"/>')
    return f"<xml{w.declarations()}>{''.join(body)}</xml>"


def model_markup(schema):
    """A whole `am3d:model3d`, with `rId2` the model and `rId3` the raster."""
    w = Writer(schema, "rId2")
    body = w.element((AM3D, "model3d"), schema.elements[(AM3D, "model3d")]["type"])
    # The raster's blip is the fallback, not the model.
    head, sep, tail = body.partition("<am3d:raster")
    tail = tail.replace('r:embed="rId2"', 'r:embed="rId3"', 1)
    return head + sep + tail, w


def anim_extensions(schema):
    w = Writer(schema, "rId2")
    items = "".join(f'<p:ext uri="{{fixture-{q[1]}}}">{w.element(q, e["type"])}</p:ext>'
                    for q, e in sorted(schema.elements.items()) if q[0] == AM3D_ANIM)
    return items, w


def picture(sid, name, rel):
    return (f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
            f'<p:blipFill><a:blip r:embed="{rel}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>'
            '<p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm>'
            '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')


def frame(sid, name, uri, data, ext=""):
    return (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="{name}"/><p:cNvGraphicFramePr/><p:nvPr/>'
            '</p:nvGraphicFramePr><p:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></p:xfrm>'
            f'<a:graphic><a:graphicData uri="{uri}">{data}</a:graphicData></a:graphic>{ext}</p:graphicFrame>')


def ole(attrs, kind, rel, img="rId5", sid=20):
    return (f'<p:oleObj {attrs} r:id="{rel}" imgW="914400" imgH="914400" progId="Package">{kind}'
            f'{picture(sid, "fallback", img)}</p:oleObj>')


def slide(body, controls=""):
    return (f'{DECL}<p:sld {NS} xmlns:mc="{MC_NS}"><p:cSld><p:spTree>{TREE}{TITLE % "Retained"}{body}</p:spTree>'
            f'{controls}</p:cSld></p:sld>')


def replace_slide(entries, xml):
    return [(n, xml if n == "ppt/slides/slide1.xml" else d) for n, d in entries]


def types(*items):
    return [(f"/{n}", t) for n, t in items]


OLE_TYPE = "application/vnd.openxmlformats-officedocument.oleObject"
PKG_TYPE = "application/vnd.openxmlformats-officedocument.package"
VML_TYPE = "application/vnd.openxmlformats-officedocument.vmlDrawing"
AX_TYPE = "application/vnd.ms-office.activeX+xml"
AX_BIN_TYPE = "application/vnd.ms-office.activeX"
GLB_TYPE = "model/gltf.binary"
ACTIVEX = ('<ax:ocx xmlns:ax="http://schemas.microsoft.com/office/2006/activeX" xmlns:r="' + R + '" '
           'ax:classid="{00000000-0000-0000-0000-000000000000}" ax:persistence="persistStreamInit" r:id="rId1"/>')


def ole_deck():
    choice = (f'<mc:AlternateContent><mc:Choice xmlns:v="{V}" Requires="v">'
              '<p:oleObj spid="_x0000_s1026" name="Sheet" showAsIcon="1" r:id="rId2" imgW="914400" imgH="914400" progId="Excel.Sheet.12">'
              '<p:embed followColorScheme="none"/></p:oleObj></mc:Choice><mc:Fallback>'
              + ole('name="Sheet"', '<p:embed followColorScheme="none"/>', "rId2") + "</mc:Fallback></mc:AlternateContent>")
    frames = [
        frame(10, "Embedded", OLE_URI, choice),
        frame(11, "Full colours", OLE_URI, ole('spid="_x0000_s1027" name="Full"', '<p:embed followColorScheme="full"><p:extLst><p:ext uri="{fixture}"><fixture:x xmlns:fixture="urn:openpresent:fixture"/></p:ext></p:extLst></p:embed>', "rId3", sid=21)),
        frame(12, "Text colours", OLE_URI, ole('name="TextBg"', '<p:embed followColorScheme="textAndBackground"/>', "rId3", sid=22)),
        frame(13, "Linked", OLE_URI, ole('name="Linked" showAsIcon="0"', '<p:link updateAutomatic="1"><p:extLst><p:ext uri="{fixture}"><fixture:x xmlns:fixture="urn:openpresent:fixture"/></p:ext></p:extLst></p:link>', "rId7", sid=23)),
    ]
    control = (f'<p:controls><mc:AlternateContent><mc:Choice xmlns:v="{V}" Requires="v">'
               '<p:control spid="1030" name="Button" r:id="rId6" imgW="914400" imgH="914400"/></mc:Choice><mc:Fallback>'
               f'<p:control name="Button" showAsIcon="0" r:id="rId6" imgW="914400" imgH="914400"><p:extLst><p:ext uri="{{fixture}}"><fixture:x xmlns:fixture="urn:openpresent:fixture"/></p:ext></p:extLst>{picture(30, "button", "rId5")}</p:control>'
               '</mc:Fallback></mc:AlternateContent>'
               f'<p:control name="Plain" r:id="rId6" imgW="1" imgH="1">{picture(31, "plain", "rId5")}</p:control></p:controls>')
    return deck(
        slide_rels=[("oleObject", "../embeddings/oleObject1.bin"), ("package", "../embeddings/package1.bin"),
                    ("vmlDrawing", "../drawings/vmlDrawing1.vml"), ("image", "../media/image1.png"),
                    ("control", "../activeX/activeX1.xml"), ("oleObject", "file:///C:/fixture/linked.xlsx", "External")],
        types_extra=types(("ppt/embeddings/oleObject1.bin", OLE_TYPE), ("ppt/embeddings/package1.bin", PKG_TYPE),
                          ("ppt/drawings/vmlDrawing1.vml", VML_TYPE), ("ppt/activeX/activeX1.xml", AX_TYPE),
                          ("ppt/activeX/activeX1.bin", AX_BIN_TYPE)),
        parts=[("ppt/embeddings/oleObject1.bin", OLE_BYTES), ("ppt/embeddings/package1.bin", OLE_BYTES + b" package"),
               ("ppt/media/image1.png", PNG), ("ppt/activeX/activeX1.xml", DECL + ACTIVEX),
               ("ppt/activeX/activeX1.bin", CONTROL_BYTES),
               ("ppt/activeX/_rels/activeX1.xml.rels", rels((ACTIVEX_BIN_REL, "activeX1.bin"))),
               ("ppt/drawings/vmlDrawing1.vml", f'<xml xmlns:v="{V}"><v:shape id="_x0000_s1026"/></xml>')]), frames, control


def vml_deck(schema):
    return deck(
        slide_rels=[("vmlDrawing", "../drawings/vmlDrawing1.vml")],
        types_extra=types(("ppt/drawings/vmlDrawing1.vml", VML_TYPE), ("ppt/media/image1.png", "image/png")),
        parts=[("ppt/drawings/vmlDrawing1.vml", vml_part(schema)), ("ppt/media/image1.png", PNG),
               ("ppt/drawings/_rels/vmlDrawing1.vml.rels", rels(("image", "../media/image1.png")))])


def model_deck(schema):
    markup, w = model_markup(schema)
    anim, wa = anim_extensions(schema)
    w.used.update(wa.used)
    declared = w.declarations()
    # Declared on the branch, so nothing outside the branch not taken has to understand them.
    alternate = (f'<mc:AlternateContent><mc:Choice{declared} Requires="am3d">'
                 + frame(10, "Model", AM3D, markup, f"<p:extLst>{anim}</p:extLst>")
                 + f'</mc:Choice><mc:Fallback>{picture(11, "Model fallback", "rId3")}</mc:Fallback></mc:AlternateContent>')
    entries = deck(
        slide_rels=[(MODEL3D_REL, "../media/model3d1.glb"), ("image", "../media/image1.png")],
        types_extra=types(("ppt/media/model3d1.glb", GLB_TYPE)),
        parts=[("ppt/media/model3d1.glb", MODEL_BYTES), ("ppt/media/image1.png", PNG)])
    return replace_slide(entries, slide(alternate))


def hostile_deck():
    """Dangling, escaping and wrong links, damaged parts, and values no object should have."""
    bad = (frame(10, "Dangling", OLE_URI, '<p:oleObj r:id="rId99" imgW="-1" imgH="x" showAsIcon="maybe" progId="javascript:alert(1)"><p:embed/></p:oleObj>')
           + frame(11, "Wrong type", OLE_URI, ole('name="w"', "<p:embed/>", "rId4", img="rId2", sid=21))
           + frame(12, "No id", OLE_URI, "<p:oleObj><p:link/><p:embed/></p:oleObj>")
           + frame(13, "Missing target", OLE_URI, ole('name="m"', "<p:embed/>", "rId3", img="rId98", sid=23))
           + frame(14, "Model out", AM3D, f'<am3d:model3d xmlns:am3d="{AM3D}" r:embed="rId5"/>'))
    controls = '<p:controls><p:control r:id="rId6"/><p:control/><fixture xmlns="urn:x"/></p:controls>'
    entries = deck(
        slide_rels=[("oleObject", "../embeddings/oleObject1.bin"), ("oleObject", "../../../../etc/passwd"),
                    ("image", "../media/none.png"), (MODEL3D_REL, "http://example.invalid/model.glb", "External"),
                    ("control", "../activeX/missing.xml"), ("vmlDrawing", "../drawings/vmlDrawing1.vml")],
        types_extra=types(("ppt/embeddings/oleObject1.bin", OLE_TYPE), ("ppt/drawings/vmlDrawing1.vml", VML_TYPE)),
        parts=[("ppt/embeddings/oleObject1.bin", OLE_BYTES),
               # Never parsed, so neither this entity nor the broken markup can do anything.
               ("ppt/drawings/vmlDrawing1.vml", '<!DOCTYPE x [<!ENTITY a SYSTEM "file:///etc/passwd">]><xml>&a;<v:shape')])
    entries = replace_slide(entries, slide(bad, controls))
    # Stored, so a test can damage the object's bytes after its checksum: the corpus tests
    # read every part of a deck they open, so the deck on disk stays readable.
    return zipped(entries, 0)


def fixtures():
    schema = audit.Schema(ROOT)
    entries, frames, control = ole_deck()
    yield "generated/area-ole.pptx", zipped(replace_slide(entries, slide("".join(frames), control))), "open CS-FND-0001 CS-FND-0007 CS-FND-0010"
    yield "generated/area-vml.pptx", zipped(vml_deck(schema)), "open"
    yield "generated/area-model3d.pptx", zipped(model_deck(schema)), "open CS-FND-0007"
    yield "hostile/retained-objects-damaged.bin", hostile_deck(), "open CS-FND-0001 CS-FND-0003 CS-FND-0004 CS-FND-0007 CS-FND-0010"
