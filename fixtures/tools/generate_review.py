#!/usr/bin/env python3
"""Tier 1M fixtures: comments, sections, zoom, classification, tags, custom XML, change
tracking, revision info, Designer, and hostile forms of each. **Synthetic:** written to the
schemas in schemas/ms-extensions and to what `fixtures/real` shows, not by PowerPoint, so
they show we follow those and keep what is there, not what Office does with it.

The parts we retain and do not model are generated from the schemas themselves by
`Instances`, one instance of each element, attribute and value, so a row of `REVIEW` or `EXT`
cannot be without a fixture for want of someone writing it out.

Imported by generate.py. The expected values are in crates/casual-slide-import/tests/review.rs
and crates/casual-slide-export/tests/review.rs.
"""
import pathlib
import xml.etree.ElementTree as ET

from generate import DECL, NS, TITLE, TREE, deck, replace, rels, seeded, zipped

ROOT = pathlib.Path(__file__).resolve().parents[2]
XS = "{http://www.w3.org/2001/XMLSchema}"
XSD_NS = XS[1:-1]
OFFICE = "http://schemas.microsoft.com/office/"
P14 = OFFICE + "powerpoint/2010/main"
P15 = OFFICE + "powerpoint/2012/main"
P188 = OFFICE + "powerpoint/2018/8/main"
P223 = OFFICE + "powerpoint/2022/03/main"
P228 = OFFICE + "powerpoint/2022/08/main"
P166 = OFFICE + "powerpoint/2016/6/main"
P16 = OFFICE + "powerpoint/2015/main"
P184 = OFFICE + "powerpoint/2018/4/main"
P202 = OFFICE + "powerpoint/2020/02/main"
P1710 = OFFICE + "powerpoint/2017/10/main"
PSLZ = OFFICE + "powerpoint/2016/slidezoom"
PSEZ = OFFICE + "powerpoint/2016/sectionzoom"
PSUZ = OFFICE + "powerpoint/2016/summaryzoom"
ACS = OFFICE + "drawing/2020/classificationShape"
PC = OFFICE + "powerpoint/2013/main/command"
AC = OFFICE + "drawing/2013/main/command"
PC2 = OFFICE + "powerpoint/2019/9/main/command"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
P = "http://schemas.openxmlformats.org/presentationml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
DS = "http://schemas.openxmlformats.org/officeDocument/2006/customXml"
SL = "http://schemas.openxmlformats.org/schemaLibrary/2006/main"
CLBL = "http://schemas.microsoft.com/office/2020/mipLabelMetadata"

# Relationship and content types: as `fixtures/real` has them, or docs/spec/parts.md.
# LabelInfo's is Inferred: no deck here holds one.
REL_AUTHORS = "http://schemas.microsoft.com/office/2018/10/relationships/authors"
REL_COMMENTS = "http://schemas.microsoft.com/office/2018/10/relationships/comments"
REL_CHANGES = "http://schemas.microsoft.com/office/2016/11/relationships/changesInfo"
REL_REVISION = "http://schemas.microsoft.com/office/2015/10/relationships/revisionInfo"
REL_LABELS = "http://schemas.microsoft.com/office/2020/02/relationships/classificationlabels"
CT_AUTHORS = "application/vnd.ms-powerpoint.authors+xml"
CT_COMMENTS = "application/vnd.ms-powerpoint.comments+xml"
CT_CHANGES = "application/vnd.ms-powerpoint.changesinfo+xml"
CT_REVISION = "application/vnd.ms-powerpoint.revisioninfo+xml"
CT_LABELS = "application/vnd.ms-office.classificationlabels+xml"
PML = "application/vnd.openxmlformats-officedocument.presentationml"
CT_LEGACY = f"{PML}.comments+xml"
CT_LEGACY_AUTHORS = f"{PML}.commentAuthors+xml"
CT_TAGS = f"{PML}.tags+xml"
CT_ITEM_PROPS = "application/vnd.openxmlformats-officedocument.customXmlProperties+xml"

MARK = "QzMarkerQz"  # document text a hostile deck carries: it must reach no finding
GUID = "{%08X-AAAA-4BBB-8CCC-000000000000}"
AUTHOR_1 = "{11111111-1111-4111-8111-111111111111}"
AUTHOR_2 = "{22222222-2222-4222-8222-222222222222}"
SECTION_1 = "{AAAAAAA1-0000-4000-8000-000000000001}"
SECTION_2 = "{AAAAAAA2-0000-4000-8000-000000000002}"


# ------------------------------------------------------------------ instances from schemas

SHAPE_MK = ' id="2" creationId="{00000000-0000-4000-8000-000000000002}"'


class Instances:
    """Instances of a global element from the vendored schemas, with every attribute and
    optional child, and enough repeats that every value and every branch of a choice occurs.
    A repeat stands as a sibling where the parent allows it, and as a further parent if not.
    A type outside Microsoft's extensions is someone else's, and takes its smallest form."""

    ANY = {  # what a moniker list holds in `fixtures/real`
        "CT_DocumentMonikerList": [("pc", "docMk", "")],
        "CT_SlideBaseMonikerList": [("pc", "docMk", ""), ("pc", "sldMk", ' cId="1" sldId="256"')],
        "CT_MainMasterMonikerList": [("pc", "docMk", ""), ("pc", "sldMasterMk", ' cId="1" sldId="2147483648"')],
        "CT_SlideLayoutMonikerList": [("pc", "docMk", ""), ("pc", "sldLayoutMk", ' cId="1" sldId="2147483649"')],
        "CT_SlideMonikerList": [("pc", "docMk", ""), ("pc", "sldMk", ' cId="1" sldId="256"')],
        "CT_CommentMonikerList": [("pc", "docMk", ""), ("pc", "cmMk", ' authorId="1" idx="1"')],
        "CT_CommentAuthorMonikerList": [("pc", "docMk", ""), ("pc", "cmAuthorMk", ' id="1"')],
        "CT_DrawingElementMonikerList": [("pc", "docMk", ""), ("ac", "spMk", SHAPE_MK)],
        "CT_ShapeMonikerList": [("pc", "docMk", ""), ("ac", "spMk", SHAPE_MK)],
        "CT_GroupShapeMonikerList": [("pc", "docMk", ""), ("ac", "grpSpMk", SHAPE_MK)],
        "CT_GraphicFrameMonikerList": [("pc", "docMk", ""), ("ac", "graphicFrameMk", SHAPE_MK)],
        "CT_ConnectorMonikerList": [("pc", "docMk", ""), ("ac", "cxnSpMk", SHAPE_MK)],
        "CT_PictureMonikerList": [("pc", "docMk", ""), ("ac", "picMk", SHAPE_MK)],
        "CT_InkMonikerList": [("pc", "docMk", ""), ("ac", "inkMk", SHAPE_MK)],
        "CT_TextCharRangeMonikerList": [("pc", "docMk", ""), ("ac", "txMk", ' cp="0" len="1"')],
        "CT_CommentV2MonikerList": [("pc", "docMk", ""), ("pc2", "cmMk", f' id="{AUTHOR_1}"')],
        "CT_CommentReplyV2MonikerList": [("pc", "docMk", ""), ("pc2", "cmRplyMk", f' id="{AUTHOR_1}"')],
    }
    VALUES = {  # lexical forms a restriction's base would not give
        "ST_Guid": "{00000000-0000-4000-8000-000000000001}", "ST_SlideId": "256",
        "ST_SlideMasterId": "2147483648", "ST_SlideLayoutId": "2147483649", "ST_CreationId": "1",
        "ST_Index": "1", "ST_Coordinate": "0", "ST_Coordinate32": "0", "ST_Percentage": "50000",
        "ST_PositiveFixedPercentage": "50000", "ST_UniversalTimeOffset": "1000", "ST_DrawingElementId": "2",
        "ST_ClientID": "{00000000-0000-4000-8000-000000000001}", "ST_ModelId": "{00000000-0000-4000-8000-000000000001}",
        "ST_RelationshipId": "rId1", "ST_Hash": "AAAA", "ST_TextAnchoringType": "t",
    }
    BUILTIN = {"boolean": "1", "dateTime": "2024-01-02T03:04:05", "string": "Fixture", "token": "fixture", "int": "1", "unsignedInt": "1",
               "unsignedByte": "1", "long": "1", "unsignedShort": "1", "short": "1", "double": "1", "hexBinary": "00", "anyURI": "urn:fixture",
               "ID": "a", "NCName": "a", "integer": "1", "byte": "1", "base64Binary": "AAAA"}
    MAX_DEPTH = 40

    def __init__(self):
        self.types, self.groups, self.attr_groups, self.elements, self.simples, self.attrs = {}, {}, {}, {}, {}, {}
        self.short = {}
        files = sorted((ROOT / "schemas/ms-extensions").glob("*.xsd")) + [ROOT / "schemas/ooxml" / n for n in (
            "dml-main.xsd", "pml.xsd", "shared-commonSimpleTypes.xsd", "shared-relationshipReference.xsd")]
        for path in files:
            self.load(path)
        # One prefix per namespace, each prefix once, whatever the schemas call them.
        spoken, used = {}, set()
        fixed = {A: "a", P: "p", R: "r", PC: "pc", AC: "ac", PC2: "pc2"}
        used |= set(fixed.values())
        for uri in sorted({k[0] for k in self.elements} | {k[0] for k in self.attrs} | set(self.short) | set(fixed)):
            prefix = fixed.get(uri) or self.short.get(uri) or "ns"
            if uri not in fixed and prefix in used:
                prefix = f"ns{len(spoken)}"
            used.add(prefix)
            spoken[uri] = prefix
        self.short = spoken

    def load(self, path):
        root = ET.parse(path).getroot()
        target = root.get("targetNamespace")
        xmlns = {}
        for _, (prefix, uri) in ET.iterparse(path, events=("start-ns",)):
            xmlns.setdefault(prefix, uri)
            if prefix and prefix != "xsd":
                self.short.setdefault(uri, prefix)
        tables = {"complexType": self.types, "simpleType": self.simples, "group": self.groups,
                  "attributeGroup": self.attr_groups, "element": self.elements, "attribute": self.attrs}
        for node in root:
            table = tables.get(node.tag.removeprefix(XS))
            if table is not None and node.get("name"):
                table[(target, node.get("name"))] = (node, xmlns, target)

    def resolve(self, name, xmlns, target):
        prefix, _, local = name.rpartition(":")
        return (xmlns.get(prefix, target) if prefix else xmlns.get("", target), local)

    def declarations(self):
        """A declaration for every prefix an instance can use, each prefix once."""
        return " ".join(f'xmlns:{p}="{u}"' for u, p in sorted(self.short.items()) if p not in ("a", "p", "r"))

    def simple_values(self, ref, xmlns, target):
        """Every value worth writing for a simple type: its enumeration, or one good lexical."""
        ns, local = self.resolve(ref, xmlns, target)
        if ns == XSD_NS:
            return [self.BUILTIN.get(local, "1")]
        if local in self.VALUES:
            return [self.VALUES[local]]
        entry = self.simples.get((ns, local))
        if entry is None:
            return ["1"]
        node, sx, st = entry
        restriction, listing, union = (node.find(XS + t) for t in ("restriction", "list", "union"))
        if listing is not None:
            return [" ".join(self.simple_values(listing.get("itemType"), sx, st))]
        if restriction is not None:
            values = [e.get("value") for e in restriction.findall(XS + "enumeration")]
            return values or self.simple_values(restriction.get("base"), sx, st)
        if union is not None and union.get("memberTypes"):
            return self.simple_values(union.get("memberTypes").split()[0], sx, st)
        return ["1"]

    def attributes_of(self, node, xmlns, target):
        """(name, values, required) of each attribute a type declares, through groups and bases."""
        out = []
        for child in node:
            tag = child.tag.removeprefix(XS)
            if tag == "attribute":
                required = child.get("use") == "required"
                ref = child.get("ref")
                if ref:
                    ns, local = self.resolve(ref, xmlns, target)
                    decl = self.attrs.get((ns, local))
                    values = self.simple_values(decl[0].get("type"), decl[1], decl[2]) if decl else ["rId1"]
                    out.append((f"{self.short.get(ns, 'r')}:{local}", values, required))
                elif child.get("type"):
                    out.append((child.get("name"), self.simple_values(child.get("type"), xmlns, target), required))
                else:
                    inner = child.find(XS + "simpleType")
                    enum = [e.get("value") for e in inner.iter(XS + "enumeration")] if inner is not None else []
                    out.append((child.get("name"), enum or ["1"], required))
            elif tag == "attributeGroup":
                group = self.attr_groups.get(self.resolve(child.get("ref"), xmlns, target))
                if group:
                    out += self.attributes_of(*group)
            elif tag in ("complexContent", "simpleContent"):
                for inner in child:
                    if inner.tag.removeprefix(XS) in ("extension", "restriction"):
                        base = self.types.get(self.resolve(inner.get("base"), xmlns, target))
                        if base:
                            out += self.attributes_of(*base)
                        out += self.attributes_of(inner, xmlns, target)
        return out

    def particles(self, node, xmlns, target):
        """The model groups a type holds, its base's first."""
        out = []
        for child in node:
            tag = child.tag.removeprefix(XS)
            if tag in ("sequence", "choice", "all", "group"):
                out.append((child, xmlns, target))
            elif tag == "complexContent":
                inner = child[0]
                base = self.types.get(self.resolve(inner.get("base"), xmlns, target))
                if base:
                    out += self.particles(*base)
                out += self.particles(inner, xmlns, target)
        return out

    def body(self, entry, stack, full):
        """Variants of what a model group holds: the first with everything it can."""
        node, xmlns, target = entry
        tag = node.tag.removeprefix(XS)
        if tag == "group":
            group = self.groups.get(self.resolve(node.get("ref"), xmlns, target))
            if not group or len(group[0]) == 0:
                return [""]
            return self.body((group[0][0], group[1], group[2]), stack, full)
        if tag == "any":
            return [""]
        if tag == "element":
            optional = node.get("minOccurs") == "0"
            if optional and not full:
                return [""]
            made = self.of_element(node, xmlns, target, stack, full)
            if not made:
                return [""]
            return ["".join(made)] if node.get("maxOccurs") == "unbounded" else made
        kids = [c for c in node if c.tag.removeprefix(XS) in ("element", "group", "sequence", "choice", "any")]
        if tag == "choice":
            out = []
            for kid in kids if full else kids[:1]:
                out += [v for v in self.body((kid, xmlns, target), stack, full) if v]
            if node.get("maxOccurs") == "unbounded":
                return ["".join(out)]
            return out or [""]
        first, extras = [], []
        for kid in kids:
            variants = self.body((kid, xmlns, target), stack, full)
            first.append(variants[0] if variants else "")
            extras.append(variants[1:])
        out = ["".join(first)]
        for i, more in enumerate(extras):
            for variant in more:
                minimal = []
                for j, kid in enumerate(kids):
                    if i == j:
                        minimal.append(variant)
                    else:
                        low = self.body((kid, xmlns, target), stack, False)
                        minimal.append(low[0] if low else "")
                out.append("".join(minimal))
        return out

    def of_element(self, node, xmlns, target, stack, full):
        ref = node.get("ref")
        if ref:
            ns, local = self.resolve(ref, xmlns, target)
            found = self.elements.get((ns, local))
            return self.instances(ns, local, *found, stack, full) if found else []
        return self.instances(target, node.get("name"), node, xmlns, target, stack, full)

    def instances(self, ns, local, node, xmlns, target, stack, full):
        typed = node.get("type")
        tkey = self.resolve(typed, xmlns, target) if typed else None
        content = self.types.get(tkey) if tkey else None
        if not typed and node.find(XS + "complexType") is not None:
            content = (node.find(XS + "complexType"), xmlns, target)
        name = f"{self.short[ns]}:{local}"
        if tkey and (tkey in stack or len(stack) > self.MAX_DEPTH):
            return []
        full = full and (tkey is None or tkey[0].startswith(OFFICE))
        if content is None:
            text = self.simple_values(typed, xmlns, target)[0] if typed else ""
            return [f"<{name}>{text}</{name}>"]
        inner_stack = stack | {tkey} if tkey else stack
        cnode, cx, ct = content
        attrs = self.attributes_of(cnode, cx, ct)
        if not full:
            attrs = [a for a in attrs if a[2]]
        variants = [""]
        for particle in self.particles(cnode, cx, ct)[:1]:
            variants = self.body(particle, inner_stack, full)
        anys = self.ANY.get(tkey[1]) if tkey else None
        if anys:
            variants = ["".join(f"<{p}:{l}{a}/>" for p, l, a in anys)]
        out = []
        for i in range(max([len(a[1]) for a in attrs] + [len(variants), 1])):
            text = "".join(f' {n}="{vals[i % len(vals)]}"' for n, vals, _ in attrs)
            kids = variants[i % len(variants)]
            out.append(f"<{name}{text}>{kids}</{name}>" if kids else f"<{name}{text}/>")
        return out

    def of(self, uri, local):
        """Every instance a global element needs."""
        node, xmlns, target = self.elements[(uri, local)]
        return self.instances(uri, local, node, xmlns, target, frozenset(), True)


# ------------------------------------------------------------------ comments

BODY_TEXT = '<a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>%s</a:t></a:r></a:p>'
MENTION = (f'<a:bodyPr/><a:lstStyle/><a:p><a:fld id="{AUTHOR_2}" type="mention"><a:rPr lang="en-US"/><a:t>@Bo</a:t></a:fld>'
           '<a:r><a:rPr lang="en-US"/><a:t> please look</a:t></a:r></a:p>')
PC_NS = f'xmlns:pc="{PC}"'
AC_NS = f'xmlns:ac="{AC}"'
SLD_MK = '<pc:sldMkLst><pc:docMk/><pc:sldMk cId="3894847844" sldId="256"/></pc:sldMkLst>'
DE_MK = ('<ac:deMkLst><pc:docMk/><pc:sldMk cId="3894847844" sldId="256"/>'
         '<ac:spMk id="2" creationId="{00000000-0000-4000-8000-000000000002}"/></ac:deMkLst>')
TX_MK = ('<ac:txMkLst><pc:docMk/><pc:sldMk cId="3894847844" sldId="256"/>'
         '<ac:spMk id="2" creationId="{00000000-0000-4000-8000-000000000002}"/><ac:txMk cp="0" len="5"/></ac:txMkLst>')

LEGACY_AUTHORS = (f'{DECL}<p:cmAuthorLst {NS}>'
                  '<p:cmAuthor id="1" name="Ann Lee" initials="AL" lastIdx="2" clrIdx="0"><p:extLst><p:ext uri="{19B8F6BF-5375-455C-9EA6-DF929625EA0E}">'
                  f'<p15:presenceInfo xmlns:p15="{P15}" userId="S::ann@example.org::1" providerId="AD"/></p:ext></p:extLst></p:cmAuthor>'
                  '<p:cmAuthor id="2" name="Bo Chan" initials="BC" lastIdx="1" clrIdx="1"/></p:cmAuthorLst>')
LEGACY_SLIDE1 = (f'{DECL}<p:cmLst {NS}>'
                 '<p:cm authorId="1" dt="2024-03-17T08:18:03.526" idx="1"><p:pos x="10" y="20"/><p:text>First note</p:text>'
                 '<p:extLst><p:ext uri="{C676402C-5697-4E1C-873F-D02D1690AC5C}">'
                 f'<p15:threadingInfo xmlns:p15="{P15}" timeZoneBias="-330"/></p:ext></p:extLst></p:cm>'
                 '<p:cm authorId="2" dt="2024-03-17T09:00:00.000" idx="1"><p:pos x="10" y="20"/><p:text>A reply</p:text>'
                 '<p:extLst><p:ext uri="{C676402C-5697-4E1C-873F-D02D1690AC5C}">'
                 f'<p15:threadingInfo xmlns:p15="{P15}" timeZoneBias="-330"><p15:parentCm authorId="1" idx="1"/></p15:threadingInfo></p:ext></p:extLst></p:cm>'
                 '</p:cmLst>')
LEGACY_SLIDE2 = f'{DECL}<p:cmLst {NS}><p:cm authorId="1" idx="2"><p:pos x="0" y="0"/><p:text>Undated note</p:text></p:cm></p:cmLst>'


def legacy_deck():
    entries = deck(2, types_extra=[("/ppt/commentAuthors.xml", CT_LEGACY_AUTHORS), ("/ppt/comments/comment1.xml", CT_LEGACY),
                                   ("/ppt/comments/comment2.xml", CT_LEGACY)],
                   parts=[("ppt/commentAuthors.xml", LEGACY_AUTHORS), ("ppt/comments/comment1.xml", LEGACY_SLIDE1),
                          ("ppt/comments/comment2.xml", LEGACY_SLIDE2)], title="Legacy")
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"), ("slide", "slides/slide2.xml"),
        ("commentAuthors", "commentAuthors.xml")))
    for i in (1, 2):
        entries = replace(entries, f"ppt/slides/_rels/slide{i}.xml.rels", rels(
            ("slideLayout", "../slideLayouts/slideLayout1.xml"), ("comments", f"../comments/comment{i}.xml")))
    return entries


MODERN_AUTHORS = (f'{DECL}<p188:authorLst xmlns:a="{A}" xmlns:r="{R}" xmlns:p188="{P188}">'
                  f'<p188:author id="{AUTHOR_1}" name="Ann Lee" initials="AL" userId="S::ann@example.org::1" providerId="AD"/>'
                  f'<p188:author id="{AUTHOR_2}" name="Bo Chan" userId="b0" providerId="Windows Live"/></p188:authorLst>')
REACTION = ('<p188:extLst><p:ext uri="{6F9A3E12-4B5D-4C7E-8F01-23456789ABCD}">'
            f'<p223:reactions xmlns:p223="{P223}"><p223:rxn type="&#x1F44D;">'
            f'<p223:instance time="2024-03-17T10:00:00" authorId="{AUTHOR_2}"/></p223:rxn></p223:reactions></p:ext></p188:extLst>')
TASK_EVENT = (f'<p228:event time="2024-03-17T10:00:00" id="{GUID % 1}"><p228:atrbtn authorId="{AUTHOR_1}"/>'
              f'<p228:anchr><p228:comment id="{GUID % 2}"/></p228:anchr>%s</p228:event>')
TASK = ('<p188:extLst><p:ext uri="{7A0B4F23-5C6E-4D8F-9012-3456789ABCDE}">'
        f'<p228:taskDetails xmlns:p228="{P228}"><p228:history>'
        + "".join(TASK_EVENT % action for action in (
            f'<p228:asgn authorId="{AUTHOR_2}"/>', '<p228:add/>', '<p228:title val="Check the numbers"/>',
            '<p228:date stDt="2024-03-17T00:00:00" endDt="2024-03-20T00:00:00"/>', '<p228:pcntCmplt val="50000"/>',
            '<p228:unasgnAll/>', f'<p228:undo id="{GUID % 3}"/>', '<p228:unknown/>'))
        + '</p228:history></p228:taskDetails></p:ext></p188:extLst>')


def modern_comments():
    p = f'xmlns:a="{A}" xmlns:p="{P}" xmlns:r="{R}" xmlns:p188="{P188}"'
    cm = (lambda attrs, kids: f'<p188:cm {attrs}>{kids}</p188:cm>')
    reply = (f'<p188:reply id="{GUID % 20}" authorId="{AUTHOR_2}" status="resolved" created="2024-03-17T09:00:00.000">'
             f'<p188:txBody>{BODY_TEXT % "A reply"}</p188:txBody>{REACTION}</p188:reply>')
    return (f'{DECL}<p188:cmLst {p} {PC_NS} {AC_NS}>'
            + cm(f'id="{GUID % 10}" authorId="{AUTHOR_1}" status="active" created="2024-03-17T08:00:00.000"',
                 f'{SLD_MK}<p188:pos x="100" y="200"/><p188:replyLst>{reply}</p188:replyLst><p188:txBody>{MENTION}</p188:txBody>{REACTION}')
            + cm(f'id="{GUID % 11}" authorId="{AUTHOR_2}" status="resolved" created="2024-03-17T08:10:00.000"',
                 f'{DE_MK}<p188:txBody>{BODY_TEXT % "On a shape"}</p188:txBody>')
            + cm(f'id="{GUID % 12}" authorId="{AUTHOR_2}" status="closed" created="2024-03-17T08:20:00.000" '
                 f'startDate="2024-03-17T00:00:00" dueDate="2024-03-20T00:00:00" assignedTo="{AUTHOR_1} {AUTHOR_2}" complete="50000" title="Check the numbers"',
                 f'{TX_MK}<p188:txBody>{BODY_TEXT % "A task"}</p188:txBody>{TASK}')
            + cm(f'id="{GUID % 13}" authorId="{AUTHOR_1}" created="2024-03-17T08:30:00.000"',
                 f'<p188:unknownAnchor/><p188:txBody>{BODY_TEXT % "Lost its anchor"}</p188:txBody>')
            + '</p188:cmLst>')


def modern_deck():
    name = "ppt/comments/modernComment_100_0.xml"
    ext = f'<p:extLst><p:ext uri="{{6950BFC3-D8DA-4A85-94F7-54DA5524770B}}"><p188:commentRel xmlns:p188="{P188}" r:id="rId2"/></p:ext></p:extLst>'
    entries = deck(types_extra=[("/ppt/authors.xml", CT_AUTHORS), (f"/{name}", CT_COMMENTS)],
                   parts=[("ppt/authors.xml", MODERN_AUTHORS), (name, modern_comments())], title="Modern")
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"), (REL_AUTHORS, "authors.xml")))
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), (REL_COMMENTS, "../comments/modernComment_100_0.xml")))
    slide = f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % "Modern 1"}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{ext}</p:sld>'
    return replace(entries, "ppt/slides/slide1.xml", slide)


# ------------------------------------------------------------------ sections, zoom, classification

def section_list():
    return ('<p:ext uri="{521415D9-36F7-43E2-AB2F-B90AF26B5E84}">'
            f'<p14:sectionLst xmlns:p14="{P14}"><p14:section name="Opening" id="{SECTION_1}"><p14:sldIdLst><p14:sldId id="256"/><p14:sldId id="257"/></p14:sldIdLst></p14:section>'
            f'<p14:section name="Rest" id="{SECTION_2}"><p14:sldIdLst><p14:sldId id="258"/></p14:sldIdLst><p14:extLst><p:ext uri="{{F0000000-0000-4000-8000-000000000000}}"><p16:designElem xmlns:p16="{P16}" val="1"/></p:ext></p14:extLst></p14:section>'
            '<p14:section name="Empty" id="{AAAAAAA3-0000-4000-8000-000000000003}"><p14:sldIdLst/></p14:section></p14:sectionLst></p:ext>')


PRES_EXT = f'<p:ext uri="{{EFAFB233-063F-42B5-8137-9DF3F51BA10A}}"><p15:sldGuideLst xmlns:p15="{P15}"/></p:ext>'


def zoom_props(guid, image="preview"):
    return (f'<p166:zmPr xmlns:p166="{P166}" id="{guid}" returnToParent="1" imageType="{image}" transitionDur="1000" showBg="0">'
            '<p166:blipFill><a:blip r:embed="rId2"/><a:stretch/></p166:blipFill>'
            '<p166:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:ln w="3175"/></p166:spPr></p166:zmPr>')


def zoom_frame(sid, requires, uri, inner, kind):
    fallback = (f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="Zoom {sid}"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
                '<p:blipFill><a:blip r:embed="rId2"/><a:stretch/></p:blipFill><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')
    if kind == "group":
        fallback = f'<p:grpSp><p:nvGrpSpPr><p:cNvPr id="{sid}" name="Zoom {sid}"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:grpSp>'
    return (f'<mc:AlternateContent xmlns:mc="{MC}"><mc:Choice xmlns:{requires}="{uri}" Requires="{requires}">'
            f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="Zoom {sid}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>'
            f'<p:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></p:xfrm><a:graphic><a:graphicData uri="{uri}">{inner}</a:graphicData></a:graphic></p:graphicFrame>'
            f'</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>')


def zoom_slide():
    slide_zm = f'<pslz:sldZm><pslz:sldZmObj sldId="257" cId="42">{zoom_props(GUID % 30)}</pslz:sldZmObj></pslz:sldZm>'
    sect_zm = f'<psez:sectionZm><psez:sectionZmObj sectionId="{SECTION_2}">{zoom_props(GUID % 31, "cover")}</psez:sectionZmObj></psez:sectionZm>'
    summ_zm = ('<psuz:summaryZm>'
               f'<psuz:summaryZmObj sectionId="{SECTION_1}" title="Opening" descr="First part" offsetFactorX="10000" offsetFactorY="20000" scaleFactorX="90000" scaleFactorY="80000">{zoom_props(GUID % 32)}</psuz:summaryZmObj>'
               f'<psuz:summaryZmObj sectionId="{SECTION_2}">{zoom_props(GUID % 33)}</psuz:summaryZmObj><psuz:gridLayout/></psuz:summaryZm>')
    summ_fixed = '<psuz:summaryZm><psuz:fixedLayout/></psuz:summaryZm>'
    classified = ('<p:sp><p:nvSpPr><p:cNvPr id="9" name="Label"><a:extLst><a:ext uri="{C183D7F6-B498-43B3-948B-1728B52AA6E4}">'
                  f'<p16:designElem xmlns:p16="{P16}" val="1"/></a:ext>'
                  '<a:ext uri="{4F2E7C6D-3B1A-4D5E-9F80-1A2B3C4D5E6F}">'
                  f'<acs:classification xmlns:acs="{ACS}" classificationOutcomeType="hdr"/></a:ext></a:extLst></p:cNvPr><p:cNvSpPr/>'
                  '<p:nvPr><p:custDataLst><p:tags r:id="rId3"/></p:custDataLst><p:extLst><p:ext uri="{3A5E9C8B-7D21-4F60-8E1A-5C4B3D2E1F00}">'
                  f'<p202:designPr xmlns:p202="{P202}" edtDesignElem="1"><p202:designTagLst><p202:designTag name="a" val="b"/></p202:designTagLst></p202:designPr></p:ext>'
                  '<p:ext uri="{1B2C3D4E-5F60-4718-9A0B-1C2D3E4F5061}">'
                  f'<p184:classification xmlns:p184="{P184}" val="ftr"/></p:ext></p:extLst></p:nvPr></p:nvSpPr><p:spPr/></p:sp>')
    frames = (zoom_frame(5, "pslz", PSLZ, slide_zm, "pic") + zoom_frame(6, "psez", PSEZ, sect_zm, "pic")
              + zoom_frame(7, "psuz", PSUZ, summ_zm, "group") + zoom_frame(8, "psuz", PSUZ, summ_fixed, "group"))
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % "Zoom"}{frames}{classified}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>'


PNG = (b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89"
       b"\x00\x00\x00\rIDATx\x9cc\xf8\xcf\xc0\x00\x00\x03\x01\x01\x00\xc9\xfe\x92\xef\x00\x00\x00\x00IEND\xaeB`\x82")
TAG_PART = f'{DECL}<p:tagLst {NS}><p:tag name="PROJECT" val="Alpha"/><p:tag name="OWNER" val="Ann"/></p:tagLst>'


def presentation(slides, ext="", extra=""):
    ids = "".join(f'<p:sldId id="{256 + i}" r:id="rId{i + 3}"/>' for i in range(slides))
    tail = f"<p:extLst>{ext}</p:extLst>" if ext else ""
    return (f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst>{ids}</p:sldIdLst>'
            f'<p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>{extra}{tail}</p:presentation>')


def sections_deck():
    entries = deck(3, types_extra=[("/ppt/tags/tag1.xml", CT_TAGS)], title="Section",
                   parts=[("ppt/media/image1.png", PNG), ("ppt/tags/tag1.xml", TAG_PART)])
    entries = replace(entries, "ppt/presentation.xml", presentation(3, section_list() + PRES_EXT))
    entries = replace(entries, "ppt/slides/slide1.xml", zoom_slide())
    return replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(
        ("slideLayout", "../slideLayouts/slideLayout1.xml"), ("image", "../media/image1.png"), ("tags", "../tags/tag1.xml")))


# ------------------------------------------------------------------ tags, custom XML, labels

ITEM = f'{DECL}<note xmlns="urn:example:note"><secret>Item text</secret></note>'
ITEM_PROPS = (f'{DECL}<ds:datastoreItem ds:itemID="{{CCCCCCCC-0000-4000-8000-000000000001}}" xmlns:ds="{DS}">'
              '<ds:schemaRefs><ds:schemaRef ds:uri="urn:example:note"/><ds:schemaRef ds:uri="urn:example:other"/></ds:schemaRefs></ds:datastoreItem>')
LIBRARY = (f'{DECL}<sl:schemaLibrary xmlns:sl="{SL}"><sl:schema sl:uri="urn:example:note" sl:manifestLocation="m.xml" '
           'sl:schemaLocation="n.xsd" sl:schemaLanguage="http://www.w3.org/2001/XMLSchema"/></sl:schemaLibrary>')
LABELS = (f'{DECL}<clbl:labelList xmlns:clbl="{CLBL}"><clbl:label id="{{DDDDDDDD-0000-4000-8000-000000000001}}" enabled="1" method="Standard" '
          'siteId="{EEEEEEEE-0000-4000-8000-000000000001}" contentBits="3" removed="0"/></clbl:labelList>')
CUSTOM_PROPS = (f'{DECL}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" '
                'xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">'
                '<property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="MSIP_Label_dddddddd-0000-4000-8000-000000000002_Enabled"><vt:lpwstr>true</vt:lpwstr></property>'
                '<property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="3" name="MSIP_Label_dddddddd-0000-4000-8000-000000000002_Name"><vt:lpwstr>Confidential</vt:lpwstr></property>'
                '<property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="4" name="MSIP_Label_dddddddd-0000-4000-8000-000000000002_SiteId"><vt:lpwstr>eeeeeeee-0000-4000-8000-000000000001</vt:lpwstr></property>'
                '</Properties>')


def custom_deck():
    entries = deck(types_extra=[("/ppt/tags/tag1.xml", CT_TAGS), ("/customXml/itemProps1.xml", CT_ITEM_PROPS), ("/docMetadata/LabelInfo.xml", CT_LABELS),
                                ("/docProps/custom.xml", "application/vnd.openxmlformats-officedocument.custom-properties+xml")],
                   root_extra=[("custom-properties", "docProps/custom.xml"), (REL_LABELS, "docMetadata/LabelInfo.xml")], title="Custom",
                   parts=[("ppt/tags/tag1.xml", TAG_PART), ("customXml/item1.xml", ITEM), ("customXml/itemProps1.xml", ITEM_PROPS),
                          ("customXml/_rels/item1.xml.rels", rels(("customXmlProps", "itemProps1.xml"))), ("customXml/item2.xml", LIBRARY),
                          ("docMetadata/LabelInfo.xml", LABELS), ("docProps/custom.xml", CUSTOM_PROPS)])
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"),
        ("tags", "tags/tag1.xml"), ("customXml", "../customXml/item1.xml"), ("customXml", "../customXml/item2.xml")))
    entries = replace(entries, "ppt/presentation.xml", presentation(1, extra='<p:custDataLst><p:custData r:id="rId5"/><p:tags r:id="rId4"/></p:custDataLst>'))
    slide = f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % "Custom 1"}</p:spTree><p:custDataLst><p:tags r:id="rId2"/></p:custDataLst></p:cSld></p:sld>'
    entries = replace(entries, "ppt/slides/slide1.xml", slide)
    return replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("tags", "../tags/tag1.xml")))


# ------------------------------------------------------------------ what is retained whole

def declared(root, decls):
    """A root element with these namespaces declared on it."""
    head, sep, tail = root.partition(">")
    if head.endswith("/"):
        return f"{head[:-1]} {decls}/{sep}{tail}"
    return f"{head} {decls}{sep}{tail}"


def review_deck(inst):
    """Change-tracking parts with every construct of `REVIEW`, and a revision info part. Each
    further instance of the root stands as a part of its own, as a part has one root."""
    decls = inst.declarations()
    parts, types, links = [], [], []
    for i, root in enumerate(inst.of(PC, "chgInfo"), 1):
        name = f"ppt/changesInfos/changesInfo{i}.xml"
        parts.append((name, DECL + declared(root, decls)))
        types.append((f"/{name}", CT_CHANGES))
        links.append((REL_CHANGES, f"changesInfos/changesInfo{i}.xml"))
    revision = inst.of(OFFICE + "powerpoint/2015/10/main", "revInfo")[0]
    parts.append(("ppt/revisionInfo.xml", DECL + declared(revision, decls)))
    types.append(("/ppt/revisionInfo.xml", CT_REVISION))
    links.append((REL_REVISION, "revisionInfo.xml"))
    entries = deck(types_extra=types, parts=parts, title="Review")
    return replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"), *links))


# Global elements a change record does not reach by itself, and everything of `EXT`, `CUSTOM`
# and `COMMENT` this tier does not model. They stay in an `ext` of the slide.
RETAINED_ROOTS = [(PC, n) for n in ("cmAuthorMkLst", "cmMkLst", "docMkLst", "sldBaseMkLst", "sldLayoutMkLst", "sldMasterMkLst", "sldMkLst")] + [
    (AC, n) for n in ("deMkLst", "spMkLst", "grpSpMkLst", "graphicFrameMkLst", "cxnSpMkLst", "picMkLst", "inkMkLst", "txMkLst")] + [
    (PC2, "cmMkLst"), (PC2, "cmRplyMkLst"), (OFFICE + "powerpoint/2022/06/main/command", "cmChg"),
    (OFFICE + "drawing/2021/scriptlink", "scriptLink"), (OFFICE + "drawing/2021/livefeed", "liveFeedProps"),
    (P1710, "readonlyRecommended"),
] + [(P14, n) for n in ("vortex", "switch", "flip", "ripple", "honeycomb", "prism", "doors", "window", "ferris", "gallery", "conveyor", "pan", "glitter",
                        "warp", "flythrough", "flash", "shred", "reveal", "wheelReverse", "bmkTgt", "nvContentPartPr", "xfrm", "extLst", "media", "browseMode",
                        "laserClr", "defaultImageDpi", "discardImageEditData", "showMediaCtrls", "laserTraceLst", "creationId", "modId", "showEvtLst")] + [
    (P15, n) for n in ("prstTrans", "sldGuideLst", "notesGuideLst", "chartTrackingRefBased", "isNarration")]


def retained_deck(inst):
    decls = inst.declarations()
    items = []
    for i, (uri, local) in enumerate(RETAINED_ROOTS):
        for j, instance in enumerate(inst.of(uri, local)):
            items.append(f'<p:ext uri="{{F{i:07X}-{j:04X}-4000-8000-000000000000}}">{instance}</p:ext>')
    # Global attributes of p14, which stand on a timing node or a shape in a real deck, under
    # `mc:Ignorable`. Ignorable means here they ask nothing of a reader.
    attrs = f'xmlns:mc="{MC}" mc:Ignorable="p14" p14:bwMode="auto" p14:bounceEnd="50000" p14:presetBounceEnd="50000"'
    slide = (f'{DECL}<p:sld {NS} {decls} {attrs}><p:cSld><p:spTree>{TREE}{TITLE % "Retained"}</p:spTree></p:cSld>'
             f'<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr><p:extLst>{"".join(items)}</p:extLst></p:sld>')
    return replace(deck(title="Retained"), "ppt/slides/slide1.xml", slide)


# ------------------------------------------------------------------ hostile

def section_deck_with(section_list_xml):
    entries = deck(3)
    ext = f'<p:ext uri="{{521415D9-36F7-43E2-AB2F-B90AF26B5E84}}">{section_list_xml}</p:ext>'
    return zipped(replace(entries, "ppt/presentation.xml", presentation(3, ext)))


def bad_comments():
    p = f'xmlns:p188="{P188}" xmlns:a="{A}" xmlns:p="{P}"'
    cm = (f'{DECL}<p188:cmLst {p} {MARK}="{MARK}"><p188:{MARK}>{MARK}</p188:{MARK}>'
          f'<p188:cm id="{MARK}" authorId="{MARK}" status="{MARK}" created="{MARK}" startDate="{MARK}" dueDate="{MARK}" assignedTo="{MARK}" complete="{MARK}" title="{MARK}" unknown="{MARK}">'
          f'<p188:pos x="{MARK}"/><p188:pos/><p188:replyLst unknown="{MARK}"><p188:{MARK}/><p188:reply unknown="{MARK}" status="{MARK}"/></p188:replyLst><p188:replyLst/>'
          f'<p188:txBody/><p188:txBody/><p188:extLst><p:ext uri="{MARK}"><p223:reactions xmlns:p223="{P223}"><p223:rxn><p223:instance time="{MARK}"/><p223:{MARK}/></p223:rxn><p223:{MARK}/></p223:reactions></p:ext></p188:extLst>'
          f'<p188:extLst/><p188:{MARK}/></p188:cm>'
          f'<p188:cm id="{GUID % 1}" authorId="{AUTHOR_1}" created="2024-01-01T00:00:00"><p188:unknownAnchor/><p188:extLst><p:ext uri="x"><p228:taskDetails xmlns:p228="{P228}"><p228:history><p228:event time="{MARK}" id="{MARK}"><p228:{MARK}/></p228:event></p228:history></p228:taskDetails></p:ext></p188:extLst></p188:cm>'
          f'<p188:cm id="{GUID % 1}" authorId="{{99999999-9999-4999-8999-999999999999}}" created="2024-01-01T00:00:00"/></p188:cmLst>')
    return zipped(replace(modern_deck(), "ppt/comments/modernComment_100_0.xml", cm))


def bad_custom():
    entries = custom_deck()
    entries = replace(entries, "ppt/tags/tag1.xml", f'{DECL}<p:tagLst {NS} {MARK}="{MARK}"><p:tag name="{MARK}" unknown="{MARK}"/><p:tag val="{MARK}"/><p:{MARK}>{MARK}</p:{MARK}></p:tagLst>')
    entries = replace(entries, "customXml/itemProps1.xml", f'{DECL}<ds:datastoreItem xmlns:ds="{DS}" ds:itemID="{MARK}" {MARK}="{MARK}"><ds:schemaRefs><ds:schemaRef/><ds:{MARK}/></ds:schemaRefs><ds:schemaRefs/><ds:{MARK}/></ds:datastoreItem>')
    return zipped(replace(entries, "docMetadata/LabelInfo.xml", f'{DECL}<clbl:labelList xmlns:clbl="{CLBL}" {MARK}="{MARK}"><clbl:label id="{MARK}" enabled="{MARK}" method="{MARK}" unknown="{MARK}"/><clbl:{MARK}/></clbl:labelList>'))


def one_comment(extension):
    return zipped(replace(modern_deck(), "ppt/comments/modernComment_100_0.xml",
        f'{DECL}<p188:cmLst xmlns:p188="{P188}" xmlns:p="{P}"><p188:cm id="{GUID % 1}" authorId="{AUTHOR_1}" created="2024-01-01T00:00:00"><p188:unknownAnchor/>{extension}</p188:cm></p188:cmLst>'))


def hostile():
    # Each a part over its own bound. The deck opens, and the part is set aside and named.
    many = seeded('<p:cm authorId="1" idx="1"><p:pos x="0" y="0"/><p:text>x</p:text></p:cm>', 100_001)
    yield "hostile/comment-limit-comments.bin", zipped(replace(legacy_deck(), "ppt/comments/comment1.xml", f"{DECL}<p:cmLst {NS}>{many}</p:cmLst>")), "open"
    authors = seeded('<p:cmAuthor id="1" name="a" initials="a" lastIdx="0" clrIdx="0"/>', 100_001)
    yield "hostile/comment-limit-authors.bin", zipped(replace(legacy_deck(), "ppt/commentAuthors.xml", f"{DECL}<p:cmAuthorLst {NS}>{authors}</p:cmAuthorLst>")), "open"
    replies = seeded(f'<p188:reply id="{GUID % 1}" authorId="{AUTHOR_1}" created="2024-01-01T00:00:00"/>', 100_001)
    yield "hostile/comment-limit-replies.bin", zipped(replace(modern_deck(), "ppt/comments/modernComment_100_0.xml",
        f'{DECL}<p188:cmLst xmlns:p188="{P188}" xmlns:a="{A}"><p188:cm id="{GUID % 1}" authorId="{AUTHOR_1}" created="2024-01-01T00:00:00"><p188:unknownAnchor/><p188:replyLst>{replies}</p188:replyLst></p188:cm></p188:cmLst>')), "open"
    tags = seeded('<p:tag name="a" val="b"/>', 100_001)
    yield "hostile/tags-limit-tags.bin", zipped(replace(custom_deck(), "ppt/tags/tag1.xml", f"{DECL}<p:tagLst {NS}>{tags}</p:tagLst>")), "open"
    sects = seeded(f'<p14:section name="s" id="{SECTION_1}"><p14:sldIdLst/></p14:section>', 10_001)
    yield "hostile/section-limit-sections.bin", section_deck_with(f'<p14:sectionLst xmlns:p14="{P14}">{sects}</p14:sectionLst>'), "open"
    entries = seeded('<p14:sldId id="256"/>', 100_001)
    yield "hostile/section-limit-slides.bin", section_deck_with(
        f'<p14:sectionLst xmlns:p14="{P14}"><p14:section name="s" id="{SECTION_1}"><p14:sldIdLst>{entries}</p14:sldIdLst></p14:section></p14:sectionLst>'), "open"
    instances = seeded(f'<p223:instance time="2024-01-01T00:00:00" authorId="{AUTHOR_1}"/>', 10_001)
    yield "hostile/comment-limit-reactions.bin", one_comment(
        f'<p188:extLst><p:ext uri="{{00000000-0000-4000-8000-000000000001}}"><p223:reactions xmlns:p223="{P223}"><p223:rxn type="a">{instances}</p223:rxn></p223:reactions></p:ext></p188:extLst>'), "open"
    events = seeded(f'<p228:event time="2024-01-01T00:00:00" id="{GUID % 1}"><p228:add/></p228:event>', 100_001)
    yield "hostile/comment-limit-task-events.bin", one_comment(
        f'<p188:extLst><p:ext uri="{{00000000-0000-4000-8000-000000000002}}"><p228:taskDetails xmlns:p228="{P228}"><p228:history>{events}</p228:history></p228:taskDetails></p:ext></p188:extLst>'), "open"
    monikers = seeded('<pc:docMk/>', 1_025)
    yield "hostile/comment-limit-monikers.bin", zipped(replace(modern_deck(), "ppt/comments/modernComment_100_0.xml",
        f'{DECL}<p188:cmLst xmlns:p188="{P188}" {PC_NS}><p188:cm id="{GUID % 1}" authorId="{AUTHOR_1}" created="2024-01-01T00:00:00"><pc:sldMkLst>{monikers}</pc:sldMkLst></p188:cm></p188:cmLst>')), "open"
    marks = seeded('<p:tags r:id="rId3"/>', 100_001)
    yield "hostile/marks-limit-part.bin", zipped(replace(deck(), "ppt/slides/slide1.xml",
        f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}</p:spTree><p:custDataLst>{marks}</p:custDataLst></p:cSld></p:sld>')), "open"
    refs = seeded('<ds:schemaRef ds:uri="urn:a"/>', 10_001)
    yield "hostile/custom-limit-schema-refs.bin", zipped(replace(custom_deck(), "customXml/itemProps1.xml",
        f'{DECL}<ds:datastoreItem xmlns:ds="{DS}" ds:itemID="{{CCCCCCCC-0000-4000-8000-000000000001}}"><ds:schemaRefs>{refs}</ds:schemaRefs></ds:datastoreItem>')), "open"
    labels = seeded('<clbl:label enabled="1"/>', 1_025)
    yield "hostile/custom-limit-labels.bin", zipped(replace(custom_deck(), "docMetadata/LabelInfo.xml",
        f'{DECL}<clbl:labelList xmlns:clbl="{CLBL}">{labels}</clbl:labelList>')), "open"
    # Damage. Everything is reported and nothing refused, and no marker reaches a finding.
    yield "hostile/comment-bad-values.bin", bad_comments(), "open"
    yield "hostile/section-bad-values.bin", section_deck_with(
        f'<p14:sectionLst xmlns:p14="{P14}" {MARK}="{MARK}"><p14:section id="{MARK}" name="{MARK}" unknown="{MARK}"><p14:sldIdLst><p14:sldId id="{MARK}"/><p14:sldId id="999"/><p14:sldId id="256"/><p14:sldId id="256"/><p14:unknown/></p14:sldIdLst><p14:sldIdLst/></p14:section>'
        f'<p14:section name="{MARK}" id="{SECTION_1}"/><p14:section name="{MARK}" id="{SECTION_1}"/><p14:unknown>{MARK}</p14:unknown></p14:sectionLst>'), "open"
    yield "hostile/custom-bad-values.bin", bad_custom(), "open"


def fixtures():
    inst = Instances()
    yield "generated/comments-legacy.pptx", zipped(legacy_deck()), "open"
    yield "generated/comments-modern.pptx", zipped(modern_deck()), "open"
    yield "generated/sections-zoom-labels.pptx", zipped(sections_deck()), "open"
    yield "generated/custom-tags-labels.pptx", zipped(custom_deck()), "open"
    yield "generated/review-every-change.pptx", zipped(review_deck(inst)), "open"
    yield "generated/extensions-retained.pptx", zipped(retained_deck(inst)), "open"
    yield from hostile()
