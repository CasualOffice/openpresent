#!/usr/bin/env python3
"""SPEC-01: audit the spec checklist against the Rust source and against the fixtures.

Two independent questions, per row of docs/spec/checklist.tsv:

  Code      Do crates/casual-slide-{import,export,model}/src name this construct?
  Fixtures  Does any fixture contain it, by a schema-directed walk of every XML part?

This reads source text, not behaviour. A reference means the code names the construct,
not that it handles it correctly. Each reference says how much can be said about it:

  context    named where the owning element is determinable, and it is an owner the
             schema allows (reading `x` inside the arm for <a:off>, called from <a:xfrm>)
  name       named, but no owning element can be determined from the source
  ident      model crate only: an identifier normalises to the name. The weakest signal
  elsewhere  named only under owners the schema does not allow: reading `ref` on
             <mergeCell> does not cover `ref` on <hyperlink>. Not coverage
  none       not named

"Owning element" is a may-analysis over match arms, comparisons, element-like calls and
call sites. It over-approximates: a helper called from two arms counts for both.

Gates (--check), STAT-01. `y` means done with a test, so a mark is held to what this tool
can check, and what it cannot check is left blank by --propose rather than assumed:

  I, E  a `context` reference in that crate (or a `spec-audit` comment on a wildcard), and
        that crate's tests naming the construct. `name` is not enough: an owner that cannot
        be determined is the same string read for something else.
        A row written through a name made at run time says so with a `// spec-audit: <row
        id>` comment, and a test must still name it.
        A helper that reads or writes a known element's attributes says so with a
        `// spec-audit: owner=bodyPr,lvl1pPr` comment before its `fn`, which stands for the
        call site the audit cannot follow. `Node::a("x")` and `Node::new(space, "x")` in the
        exporter name their element for the rest of the chain.
  Tests A test file that reads docs/spec/checklist.tsv and names an area in a literal walks
        that area's element, attribute and value rows, so it names them. The exporter's tests
        read back all they write, so they name a row for the importer too.
  Values An enum value is read and written by its enum, generically, so no crate spells it.
        I or E: the value is in the enum whose doc comment names the row's simple type, an
        attribute of that type has `context` in that crate, and that crate's tests name the
        value, the type or the enum. A test that reads the values from the checklist names
        the type.
  M     a declaration with the row's name. An enum value must be in the enum whose doc
        comment names the row's simple type. An attribute or element must be a public
        field of the struct for the row's owner (named alike, or in OWNER_STRUCTS), and not
        a name too short or generic to mean anything; or a field or variant whose doc
        comment names it, in backticks, in a struct or enum whose own doc comment names the
        row's owner, or in a variant documented as an element that hosts the owner. The
        model's or import's tests must name it.
  All   a fixture that exercises the row, by the schema-directed walk (docs/51: "A row with
        no fixture cannot be y"). Hostile decks do not count.
  Types A type, group or attribute group marked `y` needs everything it declares to meet
        the same bar, through the groups it includes. A wildcard or base row in it blocks
        it. A simple type needs a value row for each value, so only M can apply. extLst is
        retained, not read: code that names it only keeps it in order, so it never earns `y`.
        --propose marks it `r` where a fixture holds it.

Other marks, gated the same way (CLOSE-02):

  r     retained verbatim: the node stays in the part's tree through any edit, and an import
        report names it. Needs a fixture. Every counted deck goes through RETENTION_TESTS, which
        must exist. M=r forbids y in I or E. A type or group at r needs all it declares marked.
  n/a   does not apply. Only where the tool can see that: a base row (it has nothing of its own),
        I or E on a simple type (values are the model's), or a row nothing in the schemas can
        reach, so no legal deck can have it. Anything else at n/a fails.
  .     in a gated tier, blank needs a line in docs/spec/open-rows.tsv, naming a tracker row in
        docs/14 and the reason. A blank with no reason fails, a reason on a marked row fails, and
        a type or group is accounted for by a blank member that is.

Rows of content types and relationships are read by the part graph, so casual-slide-ooxml counts
as their importer and model. A `// spec-audit: <row id>` comment directly above a public model
field declares the row, for a field not named alike. Neither a comment nor an owner is enough
alone: a mark still needs the literal in the code, a test and a fixture, and a comment naming no
schema element or no row fails.

Rows starting `mc:` are hand-written (SPEC-08): Part 3 ships no XSD. They are held to the
same bar, in the one column that applies: I means the resolver in casual-slide-ooxml's
mce.rs names it, that crate's tests name it, and a fixture has it.

Also runs count.py (SPEC-03), and fails if this tool's own reading of the schemas yields
different row ids from the checklist. R and U name render and UI code outside these
crates, so they get the fixture gate only and are listed as unaudited.

None of this reads behaviour: a mark says the code names the construct where the schema
puts it, a test names it and a fixture has it. It can still be wrong, and a false `y` is
worse than a blank, so --propose takes the narrow reading everywhere.

    python3 tools/spec-audit/audit.py                # coverage table and gap summary
    python3 tools/spec-audit/audit.py --check        # gates, non-zero on failure
    python3 tools/spec-audit/audit.py --gaps FILE    # every row and its result, as TSV
    python3 tools/spec-audit/audit.py --json FILE    # machine-readable summary
    python3 tools/spec-audit/audit.py --propose 1A,1B,1C,1D          # what would be marked
    python3 tools/spec-audit/audit.py --propose 1A,1B,1C,1D --apply  # write those marks
"""
import argparse
import csv
import json
import pathlib
import re
import sys
import xml.etree.ElementTree as ET
import xml.parsers.expat
import zipfile
from collections import Counter, defaultdict

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import count as spec_count  # noqa: E402  (SPEC-03 lives beside this file)

XS = "{http://www.w3.org/2001/XMLSchema}"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
XML_NS = "http://www.w3.org/XML/1998/namespace"
CRATES = {"I": "casual-slide-import", "E": "casual-slide-export", "M": "casual-slide-model"}
RESOLVER_CRATE = "casual-slide-ooxml"
GENERATED_SOURCES = ("schema_names.rs",)
OPC_KEYS = frozenset(("ct", "rel"))
OPC_CRATE = "casual-slide-ooxml"
OPEN_ROWS = "docs/spec/open-rows.tsv"
TRACKER = "docs/14-EXECUTION-TRACKER.md"
# The tests that make `r` true for every counted deck. A mark at r is void without them.
RETENTION_TESTS = (
    ("casual-slide-export/tests/retention.rs", "every_node_of_every_deck_survives_an_edit"),
    ("casual-slide-import/tests/report.rs", "an_unread_attribute_or_element_is_always_reported"),
)
DEFAULT_GATE = "1A,1B,1C,1D"
TIERS = {"1A": ("DSIG", "PKG", "SIMPLE", "MCE"), "1B": ("PRES", "VIEW"), "1C": ("COLOR", "THEME"), "1D": ("SHAPE", "SLIDE"),
         "1E": ("GEOM", "FILL", "LINE", "EFFECT", "D3"), "1F": ("TEXT",)}
LEAF_KINDS = ("element", "attribute", "root", "rootattr", "value")
# Model fields too generic to say anything about which attribute they hold.
GENERIC_FIELDS = frozenset("id name type val value uri ref kind data text index key".split())
MIN_FIELD_NAME = 3
NEVER_EARNED = frozenset(("extLst", "ext"))
# Nothing reads inside a locked canvas: the whole of it stays in the tree, whatever the names say.
RETAINED_OWNER_PREFIX = "CT_Gvml"
# Schema type to model struct, where the two are named differently. Hand-kept, and read
# from crates/casual-slide-model/src. A type with no struct here or by name earns no M.
OWNER_STRUCTS = {
    "CT_Properties": "AppProperties",
    "AG_Locking": "Locks", "CT_ShapeLocking": "Locks", "CT_GroupLocking": "Locks", "CT_ConnectorLocking": "Locks",
    "CT_PictureLocking": "Locks", "CT_GraphicalObjectFrameLocking": "Locks", "CT_ContentPartLocking": "Locks",
    "CT_NonVisualDrawingProps": "NonVisualDrawing",
    "CT_ApplicationNonVisualDrawingProps": "AppNonVisual",
    "CT_Transform2D": "Transform",
    "CT_GroupTransform2D": "Transform",
    "CT_SlideSorterViewProperties": "SorterViewProperties",
    "CT_OutlineViewSlideEntry": "OutlineSlide",
    "CT_SlideMasterTextStyles": "MasterTextStyles",
    "CT_Slide": "SlidePart", "CT_SlideLayout": "SlidePart", "CT_SlideMaster": "SlidePart", "CT_NotesSlide": "SlidePart",
    "CT_NotesMaster": "SlidePart", "CT_HandoutMaster": "SlidePart",
    "CT_StyleMatrixReference": "StyleMatrixRef",
    "CT_FontReference": "FontRef",
    "CT_Vector": "VtVector",
    "CT_Array": "VtArray",
    "CT_Scale2D": "Scale",
    "CT_Point2D": "Point",
    "CT_PositiveSize2D": "PositiveSize",
    "CT_SlideIdListEntry": "SlideId",
    "CT_SlideMasterIdListEntry": "SlideMasterId",
    "CT_SlideLayoutIdListEntry": "SlideLayoutId",
    "CT_EmbeddedFontListEntry": "EmbeddedFont",
    "CT_ColorMapping": "ColorMap",
    "CT_ObjectStyleDefaults": "ObjectDefaults",
    "CT_OfficeStyleSheet": "Theme",
    "CT_ShapeStyle": "ShapeStyle",
    "CT_Connection": "Connection",
    "CT_BaseStylesOverride": "ThemeOverride",
    "cp:CT_Properties": "CustomProperties",
    "cp:CT_Property": "CustomProperty",
    "core:CT_CoreProperties": "CoreProperties",
    "core:CT_Keywords": "Keywords",
    "rel:CT_Relationship": "Relationship",
    "CT_Variant": "Variant",
    "CT_Presentation": "Presentation",
    "CT_PresentationProperties": "PresentationProperties",
    "CT_ViewProperties": "ViewProperties",
    "CT_Hyperlink": "Hyperlink",
    "CT_Placeholder": "Placeholder",
    "CT_HeaderFooter": "HeaderFooter",
    "EG_Background": "Background",
    "AG_ChildSlide": "SlidePart", "EG_ChildSlide": "SlidePart", "EG_TopLevelSlide": "SlidePart",
    "CT_CommonSlideData": "CommonSlideData",
    "CT_BackgroundProperties": "BackgroundProperties",
    "CT_Background": "Background",
    "CT_GroupShapeProperties": "ShapeProperties",
}
RANK = {"none": 0, "elsewhere": 1, "ident": 2, "name": 3, "context": 4, "annotated": 4}
FIXTURE_SETS = ("generated", "producers", "real", "hostile")
COUNTED_SETS = ("generated", "producers", "real")  # a hostile deck is scanned, but proves nothing about coverage
MAX_PART = 64 << 20
MAX_READ = 512 << 20
MAX_ENTRIES = 20000
MAX_DEPTH = 256
csv.field_size_limit(1 << 24)

# Conventional prefixes, so a literal "p14:media" is held to its namespace. A prefix not
# listed here compares by local name, which claims less rather than more.
ROOT_NS = {"PML": "p", "DML": "a", "REL": "r", "EP": "#ep", "CP": "#cp", "CORE": "#core"}
KNOWN_PREFIXES = {
    "#ep": "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
    "#cp": "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
    "#core": "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
    "p": "http://schemas.openxmlformats.org/presentationml/2006/main",
    "a": "http://schemas.openxmlformats.org/drawingml/2006/main",
    "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    "c": "http://schemas.openxmlformats.org/drawingml/2006/chart",
    "dgm": "http://schemas.openxmlformats.org/drawingml/2006/diagram",
    "pic": "http://schemas.openxmlformats.org/drawingml/2006/picture",
    "m": "http://schemas.openxmlformats.org/officeDocument/2006/math",
    "p14": "http://schemas.microsoft.com/office/powerpoint/2010/main",
    "p15": "http://schemas.microsoft.com/office/powerpoint/2012/main",
    "a14": "http://schemas.microsoft.com/office/drawing/2010/main",
    "a16": "http://schemas.microsoft.com/office/drawing/2014/main",
}


# ---------------------------------------------------------------- schema model

class Schema:
    """The vendored schemas, resolved by namespace, with row ids spelled as the generator spells them."""

    def __init__(self, root):
        self.types = {}       # (ns, name) -> {"key", "owner", "members", "row"}
        self.simples = {}     # (ns, name) -> {"key", "name", "row", "values", "members", "list"}
        self.elements = {}    # (ns, name) -> {"row", "type"}
        self.attributes = {}  # (ns, name) -> {"row", "type"}
        self.ids = set()
        self.ns_of_key = {}
        listed, _ = spec_count.sources(root)
        for fname, key in sorted(listed.items()):
            if (root / fname).exists():
                self._load(root / fname, key)
        self._flat = {}
        self.hosts = defaultdict(set)  # (key, owner) -> {(ns, local)}
        self.attrs_of_simple = defaultdict(set)  # simple (key, name) -> attribute local names
        self.element_names = {n for _, n in self.elements}
        self.attribute_names = {n for _, n in self.attributes}
        self._link()

    def _load(self, path, key):
        prefixes = {}
        for _, (pfx, uri) in ET.iterparse(path, events=("start-ns",)):
            prefixes[pfx] = uri
        root = ET.parse(path).getroot()
        tns = root.get("targetNamespace") or ""
        self.ns_of_key[key] = tns
        qualified = root.get("elementFormDefault") == "qualified"
        attr_qualified = root.get("attributeFormDefault") == "qualified"
        seen = Counter()

        def qname(text):
            if not text:
                return None
            pfx, _, local = text.rpartition(":")
            if pfx == "":
                return (prefixes.get("", tns), local)
            return (XML_NS if pfx == "xml" else prefixes.get(pfx, pfx), local)

        def row_id(owner, name):
            base = f"{key}:{owner}/{name}" if owner else f"{key}:{name}"
            seen[base] += 1
            rid = base if seen[base] == 1 else f"{base}#{seen[base]}"
            self.ids.add(rid)
            return rid

        def members(node, owner, out):
            for ch in node:
                tag = ch.tag.replace(XS, "")
                if tag in ("sequence", "choice", "all", "complexContent", "simpleContent"):
                    members(ch, owner, out)
                elif tag in ("extension", "restriction"):
                    base = ch.get("base")
                    if base:
                        out.append({"kind": "base", "row": row_id(owner, base.split(":")[-1]), "target": qname(base)})
                    members(ch, owner, out)
                elif tag == "element":
                    raw = ch.get("name") or ch.get("ref")
                    if ch.get("ref"):
                        q = qname(ch.get("ref"))
                    else:
                        q = (tns if qualified or ch.get("form") == "qualified" else "", ch.get("name"))
                    out.append({"kind": "element", "row": row_id(owner, raw), "q": q, "ref": bool(ch.get("ref")),
                                "type": qname(ch.get("type"))})
                elif tag == "group" and ch.get("ref"):
                    out.append({"kind": "group", "row": row_id(owner, ch.get("ref").split(":")[-1]), "target": qname(ch.get("ref"))})
                elif tag == "attribute":
                    raw = ch.get("name") or ch.get("ref")
                    if ch.get("ref"):
                        q = qname(ch.get("ref"))
                    else:
                        q = (tns if attr_qualified or ch.get("form") == "qualified" else "", ch.get("name"))
                    out.append({"kind": "attribute", "row": row_id(owner, "@" + raw), "q": q, "ref": bool(ch.get("ref")),
                                "type": qname(ch.get("type")), "local": raw.split(":")[-1]})
                elif tag == "attributeGroup" and ch.get("ref"):
                    out.append({"kind": "attrgroup", "row": row_id(owner, ch.get("ref").split(":")[-1]), "target": qname(ch.get("ref"))})
                elif tag == "any":
                    out.append({"kind": "any", "row": row_id(owner, "xsd:any")})
                elif tag == "anyAttribute":
                    out.append({"kind": "anyattr", "row": row_id(owner, "@xsd:any")})

        for node in root:
            tag, name = node.tag.replace(XS, ""), node.get("name")
            if not name:
                continue
            if tag == "element":
                row = row_id("", f"<{name}>")
                typ = qname(node.get("type"))
                inner = node.find(f"{XS}complexType")
                if inner is not None:
                    typ = ("#element", tns, name)
                    out = []
                    members(inner, name, out)
                    self.types[typ] = {"key": key, "owner": name, "members": out, "row": None}
                self.elements[(tns, name)] = {"row": row, "type": typ}
            elif tag == "attribute":
                self.attributes[(tns, name)] = {"row": row_id("", "@" + name), "type": qname(node.get("type"))}
            elif tag in ("complexType", "group", "attributeGroup"):
                row = row_id("", name)
                out = []
                members(node, name, out)
                self.types[(tns, name)] = {"key": key, "owner": name, "members": out, "row": row}
            elif tag == "simpleType":
                row = row_id("", name)
                values = {e.get("value"): row_id(name, "=" + e.get("value")) for e in node.iter(f"{XS}enumeration")}
                parts = []
                for u in node.iter(f"{XS}union"):
                    parts += [qname(m) for m in (u.get("memberTypes") or "").split()]
                for r in node.iter(f"{XS}restriction"):
                    if r.get("base"):
                        parts.append(qname(r.get("base")))
                lst = [qname(li.get("itemType")) for li in node.iter(f"{XS}list") if li.get("itemType")]
                self.simples[(tns, name)] = {"key": key, "name": name, "row": row, "values": values,
                                             "members": parts + lst, "list": bool(lst)}

    def flat(self, tkey):
        """Content of a type with groups and bases expanded. Each entry carries the use-rows on its path."""
        if tkey in self._flat:
            return self._flat[tkey]
        result = {"children": defaultdict(list), "attrs": defaultdict(list), "any": [], "anyattr": [], "simple": []}
        self._flat[tkey] = result
        t = self.types.get(tkey)
        if not t:
            return result
        for m in t["members"]:
            kind = m["kind"]
            if kind == "element":
                result["children"][m["q"]].append(((m["row"],), m))
            elif kind == "attribute":
                result["attrs"][m["q"]].append(((m["row"],), m))
            elif kind == "any":
                result["any"].append(m["row"])
            elif kind == "anyattr":
                result["anyattr"].append(m["row"])
            else:
                target = m["target"]
                if target in self.simples or target not in self.types:
                    result["simple"].append((m["row"], target))
                    continue
                sub = self.flat(target)
                extra = (m["row"],) + ((self.types[target]["row"],) if self.types[target]["row"] else ())
                for q, entries in sub["children"].items():
                    result["children"][q] += [(extra + path, d) for path, d in entries]
                for q, entries in sub["attrs"].items():
                    result["attrs"][q] += [(extra + path, d) for path, d in entries]
                result["any"] += sub["any"]
                result["anyattr"] += sub["anyattr"]
                result["simple"] += sub["simple"]
        return result

    def _link(self):
        """Which elements host each type's members, and which attributes carry each simple type."""
        uses = defaultdict(set)  # included type -> including types
        for tkey, t in self.types.items():
            for m in t["members"]:
                if m["kind"] in ("group", "attrgroup", "base") and m["target"] in self.types:
                    uses[m["target"]].add(tkey)
                if m["kind"] == "attribute":
                    self._note_simple(m["type"], m["local"])
                    if m["ref"] and m["q"] in self.attributes:
                        self._note_simple(self.attributes[m["q"]]["type"], m["local"])
        hosted = defaultdict(set)
        for q, e in self.elements.items():
            if e["type"]:
                hosted[e["type"]].add(q)
        for t in self.types.values():
            for m in t["members"]:
                if m["kind"] == "element":
                    typ = m["type"]
                    if m["ref"] and m["q"] in self.elements:
                        typ = self.elements[m["q"]]["type"]
                    if typ:
                        hosted[typ].add(m["q"])
        for tkey, t in self.types.items():
            names, todo, seen = set(), [tkey], set()
            while todo:
                cur = todo.pop()
                if cur in seen:
                    continue
                seen.add(cur)
                names |= hosted[cur]
                todo += uses[cur]
            self.hosts[(t["key"], t["owner"])] |= names
        # Local element names: a type's own elements are context too, not only global ones.
        for t in self.types.values():
            self.element_names |= {m["q"][1] for m in t["members"] if m["kind"] == "element"}
            self.attribute_names |= {m["local"] for m in t["members"] if m["kind"] == "attribute"}

    def unreachable_rows(self):
        """Rows no type in the schemas can reach: a simple type nothing is declared with, its values,
        and a global attribute nothing refers to. No legal deck can have one."""
        used, referred = set(), set()

        def use(typ):
            if not typ or typ in used or typ not in self.simples:
                return
            used.add(typ)
            for member in self.simples[typ]["members"]:
                use(member)

        for t in self.types.values():
            for m in t["members"]:
                if m["kind"] in ("attribute", "element"):
                    use(m.get("type"))
                    if m["kind"] == "attribute" and m["ref"]:
                        referred.add(m["q"])
                        if m["q"] in self.attributes:
                            use(self.attributes[m["q"]]["type"])
                elif m["kind"] == "base":
                    use(m["target"])
        for e in self.elements.values():
            use(e["type"])
        out = set()
        for q, s in self.simples.items():
            if q not in used:
                out.add(s["row"])
                out |= set(s["values"].values())
        out |= {a["row"] for q, a in self.attributes.items() if q not in referred}
        for t in self.types.values():  # a complex type no element is declared with
            if t["row"] and not self.hosts.get((t["key"], t["owner"])):
                out.add(t["row"])
                out |= {m["row"] for m in t["members"] if "row" in m}
        return out

    def _note_simple(self, typ, local, seen=None):
        seen = seen if seen is not None else set()
        if not typ or typ in seen or typ not in self.simples:
            return
        seen.add(typ)
        s = self.simples[typ]
        self.attrs_of_simple[(s["key"], s["name"])].add(local)
        for member in s["members"]:
            self._note_simple(member, local, seen)


def read_checklist(path):
    with open(path, encoding="utf8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE))


# ---------------------------------------------------------------- Rust lexer

def lex(text):
    """Tokens of Rust source: (kind, value, line), kind one of id, str, p. Comments and char
    literals vanish; `// spec-audit: id` comments come back as kind `note`."""
    toks, i, n, line = [], 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif c.isspace():
            i += 1
        elif text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            m = re.match(r"//+!?\s*spec-audit:\s*(\S+)", text[i:end])
            if m:
                toks.append(("note", m.group(1), line))
            i = end
        elif text.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    line += text[i] == "\n"
                    i += 1
        else:
            m = re.match(r'(?:b|c)?r(#*)"', text[i:i + 300])
            if m:
                close = '"' + m.group(1)
                start = i + m.end()
                end = text.find(close, start)
                end = n if end < 0 else end
                body = text[start:end]
                toks.append(("str", body, line))
                line += body.count("\n")
                i = end + len(close)
            elif c == '"' or (c in "bc" and text[i + 1:i + 2] == '"'):
                j = i + (1 if c == '"' else 2)
                out = []
                start_line = line
                while j < n and text[j] != '"':
                    if text[j] == "\\":
                        nxt = text[j + 1:j + 2]
                        if nxt == "\n":  # a line continuation swallows the following indent
                            line += 1
                            j += 2
                            while j < n and text[j].isspace():
                                line += text[j] == "\n"
                                j += 1
                            continue
                        out.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(nxt, "\\" + nxt))
                        j += 2
                    else:
                        line += text[j] == "\n"
                        out.append(text[j])
                        j += 1
                toks.append(("str", "".join(out), start_line))
                i = j + 1
            elif c == "'" or (c == "b" and text[i + 1:i + 2] == "'"):
                m = re.match(r"b?'(?:\\.[^']*|[^'\\])'", text[i:i + 16])
                if m:
                    i += m.end()
                else:  # no closing quote: a lifetime
                    i += 1
            elif c.isalpha() or c == "_":
                m = re.match(r"\w+", text[i:i + 300])
                toks.append(("id", m.group(0), line))
                i += m.end()
            elif c.isdigit():
                m = re.match(r"[\w.]+", text[i:i + 300])
                i += m.end()
            else:
                two = text[i:i + 2]
                if two in ("=>", "==", "!=", "::", "->"):
                    toks.append(("p", two, line))
                    i += 2
                else:
                    toks.append(("p", c, line))
                    i += 1
    return toks


def strip_test_items(toks, tests=None):
    """Drop whatever follows #[cfg(test)]: a test naming a construct is not an implementation of it.
    What is dropped goes into `tests`, if given."""
    out, i = [], 0
    pat = ["#", "[", "cfg", "(", "test", ")", "]"]
    while i < len(toks):
        if [t[1] for t in toks[i:i + 7]] == pat:
            i += 7
            depth = 0
            while i < len(toks):
                v = toks[i][1] if toks[i][0] == "p" else None
                if tests is not None:
                    tests.append(toks[i])
                i += 1
                if v == "{":
                    depth += 1
                elif v == "}":
                    depth -= 1
                    if depth == 0:
                        break
                elif v == ";" and depth == 0:
                    break
        else:
            out.append(toks[i])
            i += 1
    return out


# ---------------------------------------------------------------- Rust scan

ATTR_CALL = re.compile(r"attr", re.I)
ELEM_CALL = re.compile(r"child|elem|start|open|tag|find|descend|empty|begin|root|node", re.I)
BUILDER_CALLS = frozenset(("a", "new"))  # the exporter's Node::a("x"), Node::new(space, "x")
NAME_RX = re.compile(r"^(?:([A-Za-z_][\w.-]*):)?([A-Za-z_][\w.-]*)$")
VERBS = re.compile(r"^(?:read|parse|write|import|export|emit|load|save|to|from|build|serialize|deserialize|patch|sync|set)_")
TAG_RX = re.compile(r"<(/?)(?:([A-Za-z_][\w.-]*):)?([A-Za-z_][\w.-]*)((?:[^<>]|\{[^{}]*\})*)")
ATTR_RX = re.compile(r"""(?:([A-Za-z_][\w.-]*):)?([A-Za-z_][\w.-]*)=\\?["']([^"'{}]*)""")


RUST_WORDS = frozenset("as async await break const continue crate dyn else enum extern false fn for if impl in let loop match "
                       "mod move mut pub ref return self Self static struct super trait true type unsafe use where while".split())


def normal(name):
    return name.replace("_", "").replace("-", "").lower()


class Ref:
    """One string literal. ctx: elements that may own it. attr_ctx: attributes it may be a value of."""
    __slots__ = ("file", "line", "value", "prefix", "role", "ctx", "attr_ctx", "fn")

    def __init__(self, file, line, value, prefix, role, ctx, attr_ctx, fn):
        self.file, self.line, self.value, self.prefix = file, line, value, prefix
        self.role, self.ctx, self.attr_ctx, self.fn = role, ctx, attr_ctx, fn


class Frame:
    """A brace block: names established for everything inside it."""
    __slots__ = ("ctx", "attr_ctx", "fn", "calls", "arm", "arm_at")

    def __init__(self, fn):
        self.ctx, self.attr_ctx, self.fn = set(), set(), fn
        self.calls = []
        self.arm, self.arm_at = set(), None


class Scan:
    """Every string literal naming something, with its possible owners, for one crate."""

    def __init__(self, element_names):
        self.norm_elements = {normal(n): n for n in element_names}
        self.refs = []
        self.notes = set()
        self.idents = set()
        self.callers = defaultdict(list)  # fn -> [(caller fn, ctx, attr_ctx)]
        self.element_names = set(element_names)
        self.bad_owners = []  # a `spec-audit: owner=` naming no element in the schemas

    def hint(self, fn):
        return self.norm_elements.get(normal(VERBS.sub("", fn)))

    def scan_file(self, path, display, consts):
        toks = strip_test_items(lex(path.read_text(encoding="utf8", errors="replace")))
        frames = [Frame(None)]
        pend, pend_attr, stick, arm_pat = set(), set(), set(), set()
        pending_fn = None
        pending_owner = set()
        prev = None

        def visible():
            c, a = set(pend), set(pend_attr)
            for f in frames:
                c |= f.ctx | f.arm
                a |= f.attr_ctx | f.arm
            return c, a

        def record(value, prefix, role, line, ctx, attr_ctx):
            self.refs.append(Ref(display, line, value, prefix, role, frozenset(ctx), frozenset(attr_ctx), frames[-1].fn))

        def template(body, line):
            """A literal holding markup is exact about what owns what."""
            ctx, _ = visible()
            opened = []
            for m in TAG_RX.finditer(body):
                closing, pfx, local, rest = m.groups()
                if closing:
                    if opened and opened[-1] == local:
                        opened.pop()
                    continue
                record(local, pfx, "element", line, {opened[-1]} if opened else ctx, ())
                for am in ATTR_RX.finditer(rest):
                    apfx, aname, aval = am.groups()
                    if apfx == "xmlns" or aname == "xmlns":
                        continue
                    record(aname, apfx, "attribute", line, {local}, ())
                    if aval:
                        record(aval, None, "value", line, {local}, {aname})
                stick.add(local)
                if not rest.rstrip().endswith("/"):
                    opened.append(local)

        def plain(value, line, i):
            top = frames[-1]
            call = top.calls[-1] if top.calls else None
            direct = prev is not None and prev[1] in ("(", ",", "&")
            role = None
            if call and direct:
                if ATTR_CALL.search(call):
                    role = "attribute"
                elif ELEM_CALL.search(call):
                    role = "element"
                elif call in BUILDER_CALLS:
                    # Names its element for the rest of the chain, and no further.
                    pend.add(NAME_RX.match(value).group(2) if NAME_RX.match(value) else value)
            ctx, attr_ctx = visible()
            m = NAME_RX.match(value)
            prefix, local = (m.group(1), m.group(2)) if m else (None, value)
            if call in ("root", "take_ns") and direct and i >= 2 and toks[i - 2][1] in ROOT_NS:
                prefix = ROOT_NS[toks[i - 2][1]]  # r.root(ns::PML, "presentation")
            record(local, prefix, role or "any", line, ctx, attr_ctx)
            if not m:
                return
            arm = False
            for t in toks[i + 1:i + 14]:
                if t[0] == "p" and t[1] == "=>":
                    arm = True
                    break
                if not (t[0] == "str" or (t[0] == "p" and t[1] in ("|", "(", ")")) or t[1] == "Some"):
                    break
            after = toks[i + 1][1] if i + 1 < len(toks) else ""
            if arm:
                arm_pat.add(local)
            elif (prev is not None and prev[1] in ("==", "!=")) or after in ("==", "!="):
                pend.add(local)
                pend_attr.add(local)
            elif role == "attribute":
                pend_attr.add(local)
            elif role == "element":
                pend.add(local)
                stick.add(local)

        for i, (kind, value, line) in enumerate(toks):
            nxt = toks[i + 1] if i + 1 < len(toks) else ("", "", 0)
            literal = None
            if kind == "note":
                if value.startswith("owner="):
                    pending_owner = set(value.removeprefix("owner=").split(","))
                    self.bad_owners += [f"{display}:{line}: {o}" for o in sorted(pending_owner - self.element_names)]
                else:
                    self.notes.add(value)
            elif kind == "str":
                literal = value
            elif kind == "id":
                if value not in RUST_WORDS and not (prev and prev[1] in RUST_WORDS - {"pub", "let", "ref"}):
                    self.idents.add(normal(value))
                if value == "fn" and nxt[0] == "id":
                    pending_fn = nxt[1]
                elif value in consts and not (prev and prev[1] in ("const", "static")):
                    literal = consts[value]
                elif nxt[0] == "p" and (nxt[1] == "(" or (
                        nxt[1] in (",", ")") and frames[-1].calls and prev and prev[1] in ("(", ","))):
                    # `child!("showPr", show, patch_show)` calls patch_show just as a direct call would.
                    c, a = visible()
                    self.callers[value].append((frames[-1].fn, frozenset(c), frozenset(a)))
            elif kind == "p":
                top = frames[-1]
                if value == "(":
                    macro = prev and prev[1] == "!" and i >= 2 and toks[i - 2][0] == "id"  # child!("showPr", ...)
                    top.calls.append(toks[i - 2][1] if macro else (prev[1] if prev and prev[0] == "id" else None))
                elif value == ")":
                    if top.calls:
                        top.calls.pop()
                elif value == "=>":
                    if nxt[0] == "p" and nxt[1] == "{":
                        pend |= arm_pat
                        pend_attr |= arm_pat
                    else:
                        top.arm, top.arm_at = set(arm_pat), len(top.calls)
                    arm_pat = set()
                elif value == "," and top.arm_at == len(top.calls):
                    top.arm, top.arm_at = set(), None
                    pend, pend_attr, stick, arm_pat = set(), set(), set(), set()
                elif value == "{":
                    frame = Frame(pending_fn or top.fn)
                    if pending_fn:
                        h = self.hint(pending_fn)
                        if h:
                            frame.ctx.add(h)
                        frame.ctx |= pending_owner
                        pending_owner = set()
                        pending_fn = None
                    frame.ctx |= pend | stick
                    frame.attr_ctx |= pend_attr
                    frames.append(frame)
                    pend, pend_attr, stick, arm_pat = set(), set(), set(), set()
                elif value == "}":
                    if len(frames) > 1:
                        frames.pop()
                    pend, pend_attr, stick, arm_pat = set(), set(), set(), set()
                elif value == ";":
                    top.ctx |= stick
                    pend, pend_attr, stick, arm_pat = set(), set(), set(), set()
                    pending_fn = None
            if literal is not None and len(literal) < 4000:
                if re.search(r"<[A-Za-z_]", literal):
                    template(literal, line)
                else:
                    plain(literal, line, i)
            prev = (kind, value, line)

    def finish(self):
        """A reference with no owner of its own takes the owners at its function's call sites."""
        contexts, attr_contexts = defaultdict(set), defaultdict(set)
        changed = True
        while changed:
            changed = False
            for fn, sites in self.callers.items():
                for caller, ctx, attr_ctx in sites:
                    new = set(ctx) or (contexts[caller] if caller else set())
                    new_attr = set(attr_ctx) or (attr_contexts[caller] if caller else set())
                    if not new <= contexts[fn] or not new_attr <= attr_contexts[fn]:
                        contexts[fn] |= new
                        attr_contexts[fn] |= new_attr
                        changed = True
        for ref in self.refs:
            if not ref.ctx and ref.fn in contexts:
                ref.ctx = frozenset(contexts[ref.fn])
            if not ref.attr_ctx and ref.fn in attr_contexts:
                ref.attr_ctx = frozenset(attr_contexts[ref.fn])


def find_consts(paths):
    """`const NAME: &str = "x"` anywhere in the crate, so a named constant still counts as the name."""
    consts = {}
    for p in paths:
        toks = strip_test_items(lex(p.read_text(encoding="utf8", errors="replace")))
        for i, t in enumerate(toks):
            if t[0] == "id" and t[1] in ("const", "static") and i + 1 < len(toks):
                j = i + 2 if toks[i + 1][1] == "mut" else i + 1
                for k in range(j + 1, min(j + 12, len(toks))):
                    if toks[k][1] == ";":
                        break
                    if toks[k][0] == "str":
                        consts[toks[j][1]] = toks[k][1]
                        break
    return consts


def scan_crates(crates_dir, schema):
    scans, files = {}, 0
    for column, name in {**CRATES, "O": OPC_CRATE}.items():
        if not (crates_dir / name / "src").is_dir():
            continue
        scan = Scan(schema.element_names)
        # A generated table of every name is not code that names one.
        paths = sorted(p for p in (crates_dir / name / "src").rglob("*.rs") if p.name not in GENERATED_SOURCES)
        consts = find_consts(paths)
        for p in paths:
            scan.scan_file(p, str(p.relative_to(crates_dir)), consts)
            files += 1
        scan.finish()
        scans[column] = scan
    return scans, files


# ---------------------------------------------------------------- what tests and the model say

WORD = re.compile(r"[A-Za-z_][\w-]*")
ENUM_BLOCK = re.compile(r"(?:token_enum!\s*\(|xml_enum!\s*\{)\s*((?:///[^\n]*\n\s*)+)(\w+)\s*\{(.*?)\}", re.S)
STRUCT_BLOCK = re.compile(r"pub struct (\w+)\s*\{(.*?)^\}", re.S | re.M)
PUB_FIELD = re.compile(r"^\s*pub (\w+):", re.M)
ENUM_DECL = re.compile(r"pub enum (\w+)\s*\{(.*?)^\}", re.S | re.M)
VARIANT = re.compile(r"^\s*(\w+)\s*(?:\(|\{|,)", re.M)
FIELD_NOTES = re.compile(r"((?:[ \t]*//[^\n]*\n)+)[ \t]*(?:#\[[^\n]*\n[ \t]*)*(?:pub |[A-Z]\w*\s*[({])")
FIELD_NOTE_LINE = re.compile(r"//+\s*spec-audit:\s*(\S+)")
DOC_NAME = re.compile(r"`([A-Za-z_@][\w:@/.-]*)`")
CONTAINER = re.compile(r"(?:pub )?(?:struct|enum) \w+")
SCHEMA_TYPE = re.compile(r"(?:CT|EG|AG)_\w+$")


def is_test_file(path):
    return path.name == "tests.rs" or path.name.endswith("_tests.rs") or "tests" in path.parts


def tested_words(crate_dir):
    """Every word in a crate's tests, in source or in markup, normalised. Comments do not count."""
    found = set()
    for path in sorted(crate_dir.rglob("*.rs")):
        toks = lex(path.read_text(encoding="utf8", errors="replace"))
        if not is_test_file(path):
            kept = []
            strip_test_items(toks, kept)
            toks = kept
        for kind, value, _ in toks:
            if kind == "id":
                found.add(normal(value))
            elif kind == "str":
                found |= {normal(w) for w in WORD.findall(value)}
    return found


def checklist_driven(crate_dir, areas):
    """Areas a crate's tests walk row by row: a test file that reads the checklist, and names
    the area, holds each element, attribute and value row of it without spelling them."""
    found = set()
    for path in sorted(crate_dir.rglob("*")):
        if path.suffix not in (".rs", ".inc") or not is_test_file(path):
            continue
        literals = {v for kind, v, _ in lex(path.read_text(encoding="utf8", errors="replace")) if kind == "str"}
        for included in [v for v in literals if v.endswith(".inc")]:  # include!("...")
            target = path.parent / included
            if target.is_file():
                literals |= {v for kind, v, _ in lex(target.read_text(encoding="utf8", errors="replace")) if kind == "str"}
        if any("docs/spec/checklist.tsv" in v for v in literals):
            found |= literals & areas
    return found


class ModelDeclarations:
    """What the model crate declares, read from its source."""

    def __init__(self, *crate_dirs):
        self.enums = {}    # simple type name -> (enum name, its values)
        self.structs = {}  # normalised struct name -> its public fields, normalised
        self.documented = set()  # (owner or hosting element, member) from doc comments
        self.noted = set()  # row ids a `spec-audit` comment on a public field declares
        for path in sorted(p for crate_dir in crate_dirs for p in (crate_dir / "src").rglob("*.rs")):
            text = path.read_text(encoding="utf8", errors="replace")
            for block in FIELD_NOTES.findall(text):
                self.noted |= set(FIELD_NOTE_LINE.findall(block))
            for name, body in ENUM_DECL.findall(text):
                self.structs.setdefault(normal(name), set()).update(normal(v) for v in VARIANT.findall(body))
            for docs, name, body in ENUM_BLOCK.findall(text):
                named = re.search(r"\bST_\w+", docs)
                if named:
                    self.enums[named.group(0)] = (name, set(re.findall(r'=\s*"([^"]*)"', body)))
            for name, body in STRUCT_BLOCK.findall(text):
                fields = {normal(f.removeprefix("r#")) for f in PUB_FIELD.findall(body)}
                self.structs.setdefault(normal(name), set()).update(fields)
            self.read_docs(text)

    def read_docs(self, text):
        """What doc comments say each field and variant holds. A container's doc comment
        says which schema types it stands for; a member's, in backticks, which element or
        attribute it is; a variant that is an element hosts what its own fields say."""
        docs, stack = [], []
        for raw in text.splitlines():
            line = raw.strip()
            if line.startswith("///"):
                docs.append(line)
                continue
            if not line or line.startswith(("#[", "//")):
                continue
            names = [n for d in docs for n in DOC_NAME.findall(d)]
            docs = []
            opener = CONTAINER.match(line)
            if opener and line.endswith("{"):
                owners = {n.split(":")[-1] for n in names}  # a schema type, or the element it stands for
                stack.append(owners)
                continue
            if not stack:
                continue
            if line.startswith("}"):
                stack.pop()
                continue
            members = [n.split("/")[-1].lstrip("@").split(":")[-1] for n in names]
            hosts = [n.split("/")[0].lstrip("@").split(":")[-1] for n in names if "/" in n]
            for owner in stack[-1]:
                self.documented |= {(owner, m) for m in members} | {(owner, h) for h in hosts}
            for host in hosts:
                self.documented |= {(host, m) for m in members}
            if line.endswith("{"):  # a variant with fields: its doc comment names its element
                stack.append({n.split("/")[0].lstrip("@").split(":")[-1] for n in names})

    def declares(self, row, hosts=()):
        """The reason a row is in the model, or None. `hosts`: elements of the row's owner."""
        kind, name, owner = row["kind"], row["name"], row["owner"]
        rid = row.get("id", "")
        if rid in self.noted:
            return rid
        if kind == "value":
            enum = self.enums.get(owner)
            return enum[0] if enum and name[1:] in enum[1] else None
        key = rid.split(":", 1)[0]
        if kind == "root":
            if key == "vt":
                return normal(name.strip("<>")) in (self.struct_for(key, "CT_Variant") or ()) and name
            return self.struct_for(key, row["type"]) and row["type"]
        if kind not in ("element", "attribute"):
            return None
        spelled = name.lstrip("@").split(":")[-1]
        if any((where, spelled) in self.documented for where in (owner, *hosts)):
            return spelled
        bare = normal(spelled)
        if len(bare) < MIN_FIELD_NAME or bare in GENERIC_FIELDS:
            return None
        return bare if bare in (self.struct_for(key, owner) or ()) else None

    def struct_for(self, key, owner):
        """The fields of the struct for a schema type, or None if the model has no such struct."""
        name = OWNER_STRUCTS.get(f"{key}:{owner}") or OWNER_STRUCTS.get(owner, re.sub(r"^(CT|EG|AG)_", "", owner))
        return self.structs.get(normal(name))


class Evidence:
    """Everything a mark can rest on, so that proposing a mark and gating one are one rule."""

    def __init__(self, rows, keys, tiers, cov, crates_dir, schema=None):
        self.rows = {r["id"]: r for r in rows}
        self.tiers, self.cov = tiers, cov
        self.schema = schema
        self.attributes_of = defaultdict(list)  # simple type -> rows of attributes of it
        for r in rows:
            if r["kind"] == "attribute" and r["type"]:
                self.attributes_of[r["type"]].append(r["id"])
        self.model = ModelDeclarations(*[d for d in (crates_dir / CRATES["M"], crates_dir / OPC_CRATE) if (d / "src").is_dir()])
        self.opc_words = tested_words(crates_dir / OPC_CRATE) if (crates_dir / OPC_CRATE).is_dir() else set()
        self.unreachable = schema.unreachable_rows() if schema else set()
        self.users = defaultdict(list)  # simple type name -> leaf rows of that type
        for r in rows:
            if r["type"] and r["kind"] in ("attribute", "element", "rootattr"):
                self.users[r["type"].split(":")[-1]].append(r["id"])
        words = {c: tested_words(crates_dir / name) for c, name in CRATES.items()}
        # The exporter's tests read every part they write back, so they test the importer too.
        self.words = {"I": words["I"] | words["E"], "E": words["E"], "M": words["M"] | words["I"] | words["E"]}
        areas = {r["area"] for r in rows}
        driven = {c: checklist_driven(crates_dir / name, areas) for c, name in CRATES.items()}
        self.driven = {"I": driven["I"] | driven["E"], "E": driven["E"], "M": driven["M"] | driven["I"] | driven["E"]}
        self.members = defaultdict(list)
        self.declarations = {}
        for r in rows:
            if r["owner"]:
                self.members[(keys[r["id"]], r["owner"])].append(r["id"])
            if r["kind"] == "type":
                self.declarations.setdefault(r["name"], keys[r["id"]])
        self.keys = keys
        self._owned = {}

    def named_by_a_test(self, column, row):
        if row["area"] in self.driven[column] and row["kind"] in ("element", "attribute", "value", "root"):
            return True
        bare = row["name"].lstrip("=@").strip("<>").split(":")[-1]
        if normal(bare) in self.words[column]:
            return True
        if self.keys[row["id"]] in OPC_KEYS and normal(bare) in self.opc_words:
            return True
        if row["kind"] == "value" and normal(row["owner"]) in self.words[column]:
            return True  # a test that walks the type's values, by its schema name
        if row["kind"] == "value" and column == "M":
            enum = self.model.enums.get(row["owner"])
            return bool(enum) and normal(enum[0]) in self.words[column]
        return False

    def hosts_of(self, row):
        if not self.schema:
            return ()
        return tuple(local for _, local in self.schema.hosts.get((self.keys[row["id"]], row["owner"]), ()))

    def value_by_its_enum(self, column, row):
        """A value read and written by its enum: None if that holds, else why not."""
        enum = self.model.enums.get(row["owner"])
        if not enum or row["name"][1:] not in enum[1]:
            return f"no enum for {row['owner']} holds it"
        if not any(self.tiers[column].get(i, "none") == "context" for i in self.attributes_of[row["owner"]]):
            return f"no attribute of {row['owner']} has context in {CRATES[column]}"
        words = self.words[column]
        named = {normal(row["name"][1:]), normal(row["owner"]), normal(enum[0])} & words
        if not named and row["area"] not in self.driven[column]:
            return f"no test in {CRATES[column]} names it, {row['owner']} or {enum[0]}"
        return None

    def code_gap(self, column, row):
        """Why a leaf or wildcard row has not earned this column by its code and tests, or None."""
        rid = row["id"]
        if row["name"].lstrip("@").split(":")[-1] in NEVER_EARNED:
            return "retained, not read: nothing here reads it"
        if column == "M":
            if not self.model.declares(row, self.hosts_of(row)):
                return f"no declaration in {CRATES[column]} says this is the row (see the rule in audit.py)"
        elif row["kind"] == "value" and self.tiers[column].get(rid, "none") != "context":
            return self.value_by_its_enum(column, row)
        else:
            tier = self.tiers[column].get(rid, "none")
            if RANK[tier] < RANK["context"]:
                reason = {"none": "no reference", "elsewhere": "referenced only under other owners"}.get(tier, f"only `{tier}` precision")
                return f"{reason} in {CRATES[column]}"
            if tier == "annotated" and row["kind"] == "any":
                return None
        if not self.named_by_a_test(column, row):
            return f"no test in {CRATES[column]} names it"
        return None

    def owned_gap(self, column, name, key, seen):
        """The first thing a type or group declares that is not done, or None."""
        memo = (column, key, name)
        if memo in self._owned:
            return self._owned[memo]
        if memo in seen:  # a cycle adds nothing to what is being checked
            return None
        seen = seen | {memo}
        members = self.members.get((key, name), [])
        found = None if members else f"{name} declares nothing to check"
        for rid in members:
            row = self.rows[rid]
            if row["kind"] in ("group", "attrgroup"):
                found = self.included_gap(column, row["name"], key, seen)
            elif row["kind"] == "base":
                found = rid
            else:
                found = self.code_gap(column, row) and rid
            if found is None and not self.cov.hits[rid]:
                found = rid
            if found:
                break
        self._owned[memo] = found
        return found

    def included_gap(self, column, name, key, seen):
        """A group row names a declaration, in its own namespace if it has one there."""
        local = name.split(":")[-1]
        target = key if (key, local) in self.members else self.declarations.get(local, key)
        return self.owned_gap(column, local, target, seen)

    def gap(self, column, row):
        """Why a row has not earned `y` in this column, or None if it has. Fixtures are gated apart."""
        kind = row["kind"]
        key = self.keys[row["id"]]
        if kind == "type":
            missing = self.owned_gap(column, row["name"], key, frozenset())
            return f"not all it declares is done, such as {missing}" if missing else None
        if kind in ("group", "attrgroup"):
            missing = self.included_gap(column, row["name"], key, frozenset())
            return f"not all it includes is done, such as {missing}" if missing else None
        if kind == "base":
            return "a base row has nothing of its own to check"
        if kind == "simple":
            if column != "M":
                return "a simple type's values are held by the model, so only M can apply"
            values = self.members.get((key, row["name"]), [])
            if not values:
                users = [self.rows[u] for u in self.users[row["name"]] if u in self.rows]
                if any(self.model.declares(u, self.hosts_of(u)) for u in users):
                    return None
                return "a simple type with no values, and no field of the model has it"
            missing = next((i for i in values if self.code_gap("M", self.rows[i]) or not self.cov.hits[i]), None)
            return f"not all its values are done, such as {missing}" if missing else None
        return self.code_gap(column, row)

    def may_be_na(self, column, row):
        """A reason the tool can see for n/a, or None."""
        kind = row["kind"]
        if kind == "base":
            return "a base row has nothing of its own"
        if kind == "simple" and column in ("I", "E"):
            return "a simple type's values are the model's"
        if kind == "type" and not self.members.get((self.keys[row["id"]], row["name"])):
            return "a type that declares nothing has nothing to mark"
        return "nothing in the schemas reaches it" if row["id"] in self.unreachable else None

    def plain_retained(self, column, row):
        """This column's code does not touch the row: it stays in the tree, unread."""
        rid = row["id"]
        name = row["name"].lstrip("@").split(":")[-1]
        if name in NEVER_EARNED or row["kind"] == "any" or row["owner"].startswith(RETAINED_OWNER_PREFIX):
            return True
        if row["kind"] == "root" and column == "E":
            return True
        if column == "M":
            return not self.model.declares(row, self.hosts_of(row)) and self.tiers["I"].get(rid, "none") != "context"
        # With every reader and writer's owner stated, `name` is a word used for something else.
        return self.tiers[column].get(rid, "none") in ("none", "elsewhere", "name") or row["kind"] == "value"

    def declared(self, row):
        """What a type or group row declares, through the groups it includes."""
        key, seen, out = self.keys[row["id"]], set(), []
        local = row["name"].split(":")[-1]
        if row["kind"] in ("group", "attrgroup") and (key, local) not in self.members:
            key = self.declarations.get(local, key)
        todo = [(key, local)]
        while todo:
            cur = todo.pop()
            if cur in seen:
                continue
            seen.add(cur)
            for rid in self.members.get(cur, []):
                member = self.rows[rid]
                out.append(member)
                if member["kind"] in ("group", "attrgroup"):
                    sub_local = member["name"].split(":")[-1]
                    target = cur[0] if (cur[0], sub_local) in self.members else self.declarations.get(sub_local, cur[0])
                    todo.append((target, sub_local))
        return out

    def propose(self, areas, reasons=()):
        """{row id: {column: mark}} for rows still blank in those areas: `y` where earned, `n/a` where
        the tool sees it does not apply, `r` where the code does not touch it. A blank stays blank
        where code names it but something is missing, since that is work and not retention."""
        marks = {}

        def mark_of(rid, c):
            return marks.get(rid, {}).get(c) or self.rows[rid].get(c)

        def put(rid, c, value):
            if (rid, c) not in reasons:  # a recorded reason holds a row the source text alone would earn
                marks.setdefault(rid, {})[c] = value

        structural = ("type", "group", "attrgroup", "simple")
        area_rows = [r for r in self.rows.values() if r["area"] in areas]
        for row in area_rows:
            rid = row["id"]
            for c in CRATES:
                if row.get(c) != ".":
                    continue
                if self.may_be_na(c, row):
                    put(rid, c, "n/a")
                elif not self.cov.hits[rid]:
                    continue
                elif row["name"].lstrip("@").split(":")[-1] in NEVER_EARNED and row["kind"] == "element":
                    put(rid, c, "r")
                elif row["owner"].startswith(RETAINED_OWNER_PREFIX):
                    put(rid, c, "r")
                elif not self.gap(c, row):
                    put(rid, c, "y")
                elif row["kind"] not in structural and self.plain_retained(c, row):
                    put(rid, c, "r")
        # Retained in the model means not read and not written either.
        for row in area_rows:
            rid = row["id"]
            if mark_of(rid, "M") == "r":
                for c in ("I", "E"):
                    if mark_of(rid, c) == "." or marks.get(rid, {}).get(c) == "y":
                        put(rid, c, "r")
        # A type or group is retained once all it declares has a mark.
        for _ in range(4):  # a group's mark can be what a type waits for
            for row in area_rows:
                if row["kind"] not in structural or not self.cov.hits[row["id"]]:
                    continue
                for c in CRATES:
                    if mark_of(row["id"], c) != ".":
                        continue
                    if row["kind"] == "simple":
                        put(row["id"], c, "r")
                    elif self.declared(row) and all(mark_of(m["id"], c) != "." for m in self.declared(row)):
                        put(row["id"], c, "r")
        return marks


class HandWritten:
    """The `mc:` rows, which name the resolver's literals, not a schema's."""

    def __init__(self, all_rows, cov, crates_dir):
        self.rows = [r for r in all_rows if r["id"].startswith(spec_count.HAND_WRITTEN_PREFIX)]
        self.cov = cov
        crate = crates_dir / RESOLVER_CRATE
        self.literals = set()
        for path in sorted((crate / "src").rglob("*.rs")):
            toks = strip_test_items(lex(path.read_text(encoding="utf8", errors="replace")))
            self.literals |= {v for k, v, _ in toks if k == "str"}
        self.words = tested_words(crate)

    def gap(self, row):
        bare = row["name"].strip("<>@")
        if bare not in self.literals:
            return f"{RESOLVER_CRATE} does not read it"
        if normal(bare) not in self.words:
            return f"no test in {RESOLVER_CRATE} names it"
        return None if self.cov.hits[row["id"]] else "no fixture exercises it"

    def gates(self):
        failures = []
        for r in self.rows:
            for column in ("M", "I", "E", "R", "U"):
                if r.get(column) != "y":
                    continue
                reason = self.gap(r) if column == "I" else "only I applies to a hand-written row"
                if reason:
                    failures.append(f"{r['id']}: {column}=y, {reason}")
        return failures

    def propose(self, areas):
        """The resolver is the reader. The model never holds these, and they stay as written."""
        marks = {}
        for r in self.rows:
            if r["area"] not in areas:
                continue
            found = {}
            if r.get("I") == "." and not self.gap(r):
                found["I"] = "y"
            if r.get("M") == ".":
                found["M"] = "n/a"
            if r.get("E") == "." and self.cov.hits[r["id"]]:
                found["E"] = "r"
            if found:
                marks[r["id"]] = found
        return marks


def write_marks(checklist, marks):
    """Set status columns in place, changing nothing else in the file."""
    lines = checklist.read_text(encoding="utf8").split("\n")
    columns = lines[0].split("\t")
    index = {c: columns.index(c) for c in CRATES}
    for number, line in enumerate(lines[1:], 1):
        cols = line.split("\t")
        if cols[0] in marks:
            for column, value in marks[cols[0]].items():
                cols[index[column]] = value
            lines[number] = "\t".join(cols)
    checklist.write_text("\n".join(lines), encoding="utf8")


def classify(row, key, schema, idx):
    """How one leaf row is referenced in one crate's source; None for a structural row."""
    kind, name = row["kind"], row["name"]
    ns = schema.ns_of_key.get(key)
    if kind in ("element", "attribute", "root", "rootattr"):
        pfx, _, local = name.strip("<>").lstrip("@").rpartition(":")
        row_ns = KNOWN_PREFIXES.get(pfx) if pfx else ns
        want = "attribute" if kind in ("attribute", "rootattr") else "element"

        def compatible(r):
            known = KNOWN_PREFIXES.get(r.prefix) if r.prefix else None
            return r.role in (want, "any") and (known is None or row_ns is None or known == row_ns)

        refs = [r for r in idx.get(local, ()) if compatible(r)]
        if not refs:
            return "none"
        if kind in ("root", "rootattr"):
            # There is no owner to establish. A prefix pins the namespace, which is all there is to pin.
            return "context" if any(r.prefix and KNOWN_PREFIXES.get(r.prefix) == ns for r in refs) else "name"
        hosts = {n for _, n in schema.hosts.get((key, row["owner"]), ())}
        best = "none"
        for r in refs:
            known = {c for c in r.ctx if c in schema.element_names}
            if known & hosts:
                return "context"
            if not known:
                best = "name"
            elif best == "none":
                best = "elsewhere"
        return best
    if kind == "value":
        attrs = schema.attrs_of_simple.get((key, row["owner"]), set())
        best = "none"
        for r in idx.get(name[1:], ()):
            named = {a for a in r.attr_ctx if a in schema.attribute_names}
            if named & attrs:
                return "context"
            if not named:
                best = "name"
            elif best == "none":
                best = "elsewhere"
        return best
    return None


def code_coverage(rows, keys, schema, scans):
    """{column: {row id: tier}}. Structural rows take the best tier among what they declare."""
    leaf = ("element", "attribute", "value", "root", "rootattr")
    by_owner = defaultdict(list)
    by_owner_name = defaultdict(list)
    users_of_type = defaultdict(list)
    for r in rows:
        if r["owner"] and r["kind"] in leaf:
            by_owner[(keys[r["id"]], r["owner"])].append(r["id"])
            by_owner_name[r["owner"]].append(r["id"])
        if r["type"] and r["kind"] in leaf:
            users_of_type[r["type"]].append(r["id"])
    out = {}
    for column, scan in scans.items():
        idx = defaultdict(list)
        for ref in scan.refs:
            idx[ref.value].append(ref)
        tiers = {}
        for r in rows:
            tier = classify(r, keys[r["id"]], schema, idx)
            if tier is None:
                continue
            if tier == "none" and column == "M":
                bare = r["name"].lstrip("=@").strip("<>").split(":")[-1]
                if normal(bare) in scan.idents:
                    tier = "ident"
            if r["id"] in scan.notes:  # read through a name made at run time: the code says so
                tier = "annotated"
            tiers[r["id"]] = tier

        def best(ids):
            return max((tiers.get(i, "none") for i in ids), key=RANK.__getitem__, default="none")

        for r in rows:
            if r["kind"] in ("type", "simple"):
                members = by_owner[(keys[r["id"]], r["name"])] or users_of_type[r["name"]]
                tiers[r["id"]] = best(members)
            elif r["kind"] in ("base", "group", "attrgroup"):
                tiers[r["id"]] = best(by_owner[(keys[r["id"]], r["name"])] or by_owner_name[r["name"]])
            elif r["kind"] == "any":
                tiers[r["id"]] = "annotated" if r["id"] in scan.notes else "none"
        out[column] = tiers
    for rid, tier in out.pop("O", {}).items():
        if keys[rid] in OPC_KEYS and RANK[tier] > RANK[out["I"].get(rid, "none")]:
            out["I"][rid] = tier
    return out


# ---------------------------------------------------------------- fixtures

class Refused(Exception):
    pass


def refuse_dangerous(data, part):
    """The defusedxml rule, applied before any tree is built: no DOCTYPE, no entity, bounded depth.
    Expat sees the declaration whatever the encoding, and this stops there rather than expand anything."""
    depth = [0]

    def doctype(*_):
        raise Refused(f"{part}: DOCTYPE")

    def entity(*_):
        raise Refused(f"{part}: entity declaration")

    def start(*_):
        depth[0] += 1
        if depth[0] > MAX_DEPTH:
            raise Refused(f"{part}: nesting deeper than {MAX_DEPTH}")

    def end(*_):
        depth[0] -= 1

    parser = xml.parsers.expat.ParserCreate()
    parser.StartDoctypeDeclHandler = doctype
    parser.EntityDeclHandler = entity
    parser.StartElementHandler = start
    parser.EndElementHandler = end
    try:
        parser.Parse(data, True)
    except xml.parsers.expat.ExpatError as err:
        raise Refused(f"{part}: not well formed: {err}") from err


class FixtureCoverage:
    def __init__(self, schema):
        self.schema = schema
        self.hits = Counter()
        self.first = {}
        self.unmatched = Counter()
        self.refused = []
        self.parts = 0
        self.decks = 0
        self.current = ""
        self.counting = True

    def mark(self, row):
        if row and self.counting:
            self.hits[row] += 1
            self.first.setdefault(row, self.current)

    def note(self, what):
        if self.counting:
            self.unmatched[what] += 1

    def mark_simple(self, typ, value, seen=None):
        seen = seen if seen is not None else set()
        s = self.schema.simples.get(typ)
        if not s or typ in seen:
            return
        seen.add(typ)
        self.mark(s["row"])
        for token in (value.split() if s["list"] else [value]):
            if token in s["values"]:
                self.mark(s["values"][token])
            for member in s["members"]:
                self.mark_simple(member, token, seen)

    def enter(self, typ):
        t = self.schema.types.get(typ)
        if t and t["row"]:
            self.mark(t["row"])
        if typ in self.schema.simples:
            self.mark(self.schema.simples[typ]["row"])

    def global_element(self, q):
        e = self.schema.elements.get(q)
        if not e:
            return None
        self.mark(e["row"])
        return e["type"]

    def attributes(self, elem, flat, local):
        schema = self.schema
        for name, value in elem.attrib.items():
            ans, _, alocal = name[1:].rpartition("}") if name.startswith("{") else ("", "", name)
            q = (ans, alocal)
            entries = flat["attrs"].get(q)
            if entries:
                for path, decl in entries:
                    for row in path:
                        self.mark(row)
                    atype = decl["type"]
                    if decl["ref"] and q in schema.attributes:
                        self.mark(schema.attributes[q]["row"])
                        atype = atype or schema.attributes[q]["type"]
                    self.mark_simple(atype, value)
            elif flat["anyattr"]:
                for row in flat["anyattr"]:
                    self.mark(row)
            elif q in schema.attributes:  # foreign but declared globally, such as p14:dur under mc:Ignorable
                self.mark(schema.attributes[q]["row"])
                self.mark_simple(schema.attributes[q]["type"], value)
            elif ans not in (MC, XML_NS):
                self.note(f"attribute {name} on {local}")

    def walk(self, data):
        """Schema-directed, so `x` on <a:off> marks CT_Point2D/@x and nothing else. Types come from
        the declared content of the parent; mc: wrappers are transparent, and both branches count."""
        schema = self.schema
        stack = []
        pull = ET.XMLPullParser(events=("start", "end"))
        pull.feed(data)
        pull.close()
        for event, elem in pull.read_events():
            ns, _, local = elem.tag[1:].rpartition("}") if elem.tag.startswith("{") else ("", "", elem.tag)
            if event == "end":
                typ, transparent = stack.pop()
                if not transparent and typ:
                    text = (elem.text or "").strip()
                    if typ in schema.simples:
                        self.mark_simple(typ, text)
                    elif typ in schema.types:
                        for row, base in schema.flat(typ)["simple"]:
                            self.mark(row)
                            self.mark_simple(base, text)
                elem.clear()
                continue
            for name in elem.attrib:
                if name.startswith("{" + MC + "}"):
                    self.mark(f"mc:@{name.rpartition('}')[2]}")
            if ns == MC and local in ("AlternateContent", "Choice", "Fallback"):
                if local == "AlternateContent":
                    self.mark("mc:<AlternateContent>")
                else:
                    self.mark(f"mc:AlternateContent/{local}")
                    if local == "Choice" and "Requires" in elem.attrib:
                        self.mark("mc:Choice/@Requires")
                stack.append((stack[-1][0] if stack else None, True))
                continue
            q = (ns, local)
            parent = stack[-1][0] if stack else None
            typ = None
            if parent in schema.types:
                flat = schema.flat(parent)
                entries = flat["children"].get(q)
                if entries:
                    for path, decl in entries:
                        for row in path:
                            self.mark(row)
                        if decl["ref"]:
                            typ = self.global_element(q) or typ
                        elif typ is None:
                            typ = decl["type"]
                else:
                    for row in flat["any"]:
                        self.mark(row)
                    typ = self.global_element(q)
                    if q not in schema.elements or not flat["any"]:
                        self.note(f"element {{{ns}}}{local} under {parent[-1]}")
            else:
                typ = self.global_element(q)
                if q not in schema.elements:
                    self.note(f"element {{{ns}}}{local}, no schema type at this point")
            self.enter(typ)
            if typ in schema.types:
                self.attributes(elem, schema.flat(typ), local)
            stack.append((typ, False))

    def scan_deck(self, path, rel, counting=True):
        self.current = rel
        self.counting = counting
        self.decks += 1
        try:
            archive = zipfile.ZipFile(path)
        except (zipfile.BadZipFile, OSError, ValueError) as err:
            self.refused.append(f"{rel}: not a readable package: {err}")
            return
        total = 0
        with archive:
            infos = archive.infolist()
            if len(infos) > MAX_ENTRIES:
                self.refused.append(f"{rel}: {len(infos)} entries")
                return
            for info in infos:
                if info.is_dir() or not info.filename.lower().endswith((".xml", ".rels", ".vml")):
                    continue
                label = f"{rel}!{info.filename}"
                try:
                    with archive.open(info) as handle:  # the declared size is not to be believed
                        data = handle.read(MAX_PART + 1)
                except (zipfile.BadZipFile, OSError, RuntimeError, NotImplementedError, EOFError) as err:
                    self.refused.append(f"{label}: unreadable: {err}")
                    continue
                total += len(data)
                if len(data) > MAX_PART or total > MAX_READ:
                    self.refused.append(f"{label}: over the read limit")
                    continue
                self.parts += 1
                try:
                    refuse_dangerous(data, label)
                    self.walk(data)
                except Refused as err:
                    self.refused.append(str(err))
                except ET.ParseError as err:
                    self.refused.append(f"{label}: {err}")


def scan_fixtures(fixtures_dir, schema):
    cov = FixtureCoverage(schema)
    for name in FIXTURE_SETS:
        folder = fixtures_dir / name
        if not folder.is_dir():
            continue
        for path in sorted(p for p in folder.rglob("*") if p.is_file() and p.suffix.lower() not in (".tsv", ".md")):
            cov.scan_deck(path, str(path.relative_to(fixtures_dir)), counting=name in COUNTED_SETS)
    return cov


# ---------------------------------------------------------------- gates and report

def gates(rows, evidence, cov):
    failures, warnings = [], []
    for r in rows:
        rid = r["id"]
        marked = [c for c in ("M", "I", "E", "R", "U") if r.get(c) == "y"]
        for c in marked:
            if c not in CRATES:
                warnings.append(f"{rid}: {c}=y is not auditable from these crates")
                continue
            reason = evidence.gap(c, r)
            if reason:
                failures.append(f"{rid}: {c}=y, {reason}")
        if marked and cov.hits[rid] == 0:
            failures.append(f"{rid}: {','.join(marked)}=y, no fixture exercises it")
    return failures, warnings


def read_open_rows(path):
    """{(row id, column): (tracker id, reason)}, and what is wrong with the file."""
    found, problems = {}, []
    if not path.exists():
        return found, problems
    with open(path, encoding="utf8", newline="") as handle:
        for number, line in enumerate(handle.read().split("\n")[1:], 2):
            if not line.strip():
                continue
            cols = line.split("\t")
            if len(cols) != 4 or not cols[2].strip() or not cols[3].strip():
                problems.append(f"{path.name}:{number}: needs a row id, columns, tracker id and reason")
                continue
            for column in cols[1]:
                if column not in CRATES or (cols[0], column) in found:
                    problems.append(f"{path.name}:{number}: column {column} is unknown or repeated")
                found[(cols[0], column)] = (cols[2], cols[3])
    return found, problems


def status_gates(all_rows, evidence, hand_written, open_rows, areas, tracker_text, retention_present):
    """r, n/a and blanks. See the top of this file."""
    failures = []
    known = {r["id"]: r for r in all_rows}
    hand = {r["id"] for r in hand_written.rows}

    def blank_accounted(row, column, seen=()):
        if (row["id"], column) in open_rows:
            return True
        if row["id"] in seen or row["id"] in hand or row["kind"] not in ("type", "group", "attrgroup"):
            return False
        return any(m[column] == "." and blank_accounted(m, column, (*seen, row["id"])) for m in evidence.declared(row))

    for row in all_rows:
        rid = row["id"]
        for c in CRATES:
            mark = row.get(c)
            if rid in hand:
                if mark == "n/a" and c != "M":
                    failures.append(f"{rid}: {c}=n/a, only M can be n/a on a hand-written row")
            elif mark == "n/a" and not evidence.may_be_na(c, row):
                failures.append(f"{rid}: {c}=n/a, and the tool sees nothing that says so")
            if mark == "r":
                if not evidence.cov.hits[rid]:
                    failures.append(f"{rid}: {c}=r, no fixture exercises it")
                if not retention_present:
                    failures.append(f"{rid}: {c}=r, and the retention tests are missing: {RETENTION_TESTS}")
                if rid not in hand and row["kind"] in ("type", "group", "attrgroup") and any(
                        m[c] == "." for m in evidence.declared(row)):
                    failures.append(f"{rid}: {c}=r, and not all it declares has a mark")
            if mark == "." and row["area"] in areas and not blank_accounted(row, c):
                failures.append(f"{rid}: {c} is blank, with no reason in {OPEN_ROWS}")
        if row.get("M") == "r" and "y" in (row.get("I"), row.get("E")) and row["kind"] not in ("type", "group", "attrgroup"):
            failures.append(f"{rid}: M=r, so nothing reads or writes it, but I or E says y")
    for (rid, column), (tracker, _) in sorted(open_rows.items()):
        if rid not in known:
            failures.append(f"{OPEN_ROWS}: {rid} is no row")
        elif known[rid].get(column) != ".":
            failures.append(f"{OPEN_ROWS}: {rid} {column} has a reason and a mark {known[rid].get(column)}")
        if f"| {tracker} |" not in tracker_text:
            failures.append(f"{OPEN_ROWS}: {rid} names {tracker}, which is no row in docs/14")
    return failures


def retention_tests_exist(crates_dir):
    for rel, name in RETENTION_TESTS:
        path = crates_dir / rel
        if not path.exists() or f"fn {name}" not in path.read_text(encoding="utf8"):
            return False
    return True


def summary_table(rows, tiers, cov):
    areas = defaultdict(Counter)
    for r in rows:
        a = areas[r["area"]]
        a["rows"] += 1
        for col in tiers:
            a[f"{col}:{tiers[col].get(r['id'], 'none')}"] += 1
        a["fixture"] += cov.hits[r["id"]] > 0
        a["y"] += any(r.get(c) == "y" for c in ("M", "I", "E", "R", "U"))
    return areas


def print_report(rows, areas, tiers, cov, files, problems, failures, warnings, samples):
    total = Counter()
    for a in areas.values():
        total.update(a)
    print("# Spec audit (source text and fixtures; neither proves behaviour)\n")
    print(f"Scanned {files} Rust files, {cov.decks} fixture files, {cov.parts} XML parts.")
    print("String literals seen: " + ", ".join(f"{CRATES.get(c, OPC_CRATE)} {n}" for c, n in samples.items()) + ".\n")
    print("`ctx` owner determined and allowed; `name` no owner determinable; `else` only under other owners.")
    print("`Fixture` counts generated, producers and real decks; hostile decks are scanned but do not count.\n")
    print("| Area | Rows | I ctx | I name | I else | E ctx | E name | E else | M ctx | M name | M ident | Fixture | Marked y |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")

    def line(label, a):
        print(f"| {label} | {a['rows']} | {a['I:context'] + a['I:annotated']} | {a['I:name']} | {a['I:elsewhere']} | "
              f"{a['E:context'] + a['E:annotated']} | {a['E:name']} | {a['E:elsewhere']} | "
              f"{a['M:context'] + a['M:annotated']} | {a['M:name']} | {a['M:ident']} | {a['fixture']} | {a['y']} |")

    for area in sorted(areas):
        line(area, areas[area])
    line("**All**", total)
    print("\n## Gaps\n")
    print(f"Rows no fixture exercises: {total['rows'] - total['fixture']} of {total['rows']}.")
    for col in tiers:
        print(f"Rows {CRATES[col]} does not name: {total[f'{col}:none']} of {total['rows']}.")
    print("\n| Area | No fixture | Examples |")
    print("| --- | ---: | --- |")
    examples = defaultdict(list)
    for r in rows:
        if cov.hits[r["id"]] == 0 and r["kind"] in ("element", "attribute", "value", "root"):
            examples[r["area"]].append(r["id"])
    for area in sorted(areas):
        a = areas[area]
        print(f"| {area} | {a['rows'] - a['fixture']} | " + ", ".join(f"`{e}`" for e in examples[area][:3]) + " |")
    print("\n`--gaps FILE` writes every row.")
    if cov.unmatched:
        print(f"\n## In fixtures but outside the checklist: {sum(cov.unmatched.values())} uses of {len(cov.unmatched)} names\n")
        for what, n in cov.unmatched.most_common(15):
            print(f"- {n} x {what}")
    if cov.refused:
        print(f"\n## Refused without building a tree: {len(cov.refused)}\n")
        for what in cov.refused:
            print(f"- {what}")
    print("\n## Gates\n")
    for p in problems + failures:
        print(f"FAIL {p}")
    print(f"{len(failures) + len(problems)} failures, {len(warnings)} warnings.")
    for w in warnings[:30]:
        print(f"warn {w}")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    here = pathlib.Path(__file__).resolve().parents[2]
    ap.add_argument("--root", type=pathlib.Path, default=here)
    ap.add_argument("--checklist", type=pathlib.Path)
    ap.add_argument("--crates", type=pathlib.Path)
    ap.add_argument("--fixtures", type=pathlib.Path)
    ap.add_argument("--check", action="store_true", help="fail on a `y` row without a reference or a fixture")
    ap.add_argument("--propose", help="tiers, comma separated: print the marks the evidence earns in them")
    ap.add_argument("--apply", action="store_true", help="with --propose, write those marks to the checklist")
    ap.add_argument("--why", type=pathlib.Path, help="with --propose, write each mark not earned and why, as TSV")
    ap.add_argument("--gaps", type=pathlib.Path, help="write every row and its result, as TSV")
    ap.add_argument("--json", type=pathlib.Path, help="write the summary as JSON")
    ap.add_argument("--gate", default=DEFAULT_GATE, help="tiers where a blank needs a reason in " + OPEN_ROWS)
    args = ap.parse_args(argv)

    checklist = args.checklist or args.root / "docs/spec/checklist.tsv"
    all_rows = read_checklist(checklist)
    schema = Schema(args.root)
    ordered = sorted(schema.ns_of_key, key=len, reverse=True)
    keys = {r["id"]: spec_count.row_key(r["id"], ordered) for r in all_rows}
    in_list = {r["id"] for r in all_rows}
    problems = [f"schema model: {i}: from the schemas, no checklist row" for i in sorted(schema.ids - in_list)[:20]]
    from_schemas = {i for i in in_list if not i.startswith(spec_count.HAND_WRITTEN_PREFIX)}
    problems += [f"schema model: {i}: a checklist row, not from the schemas" for i in sorted(from_schemas - schema.ids)[:20]]
    rows = [r for r in all_rows if keys[r["id"]] is not None]

    scans, files = scan_crates(args.crates or args.root / "crates", schema)
    tiers = code_coverage(rows, keys, schema, scans)
    cov = scan_fixtures(args.fixtures or args.root / "fixtures", schema)
    _, _, count_problems = spec_count.compare(args.root, checklist)
    problems += [f"count: {p}" for p in count_problems]
    crates_dir = args.crates or args.root / "crates"
    evidence = Evidence(rows, keys, tiers, cov, crates_dir, schema)
    hand_written = HandWritten(all_rows, cov, crates_dir)
    if args.propose:
        wanted = args.propose.split(",")
        if unknown := [t for t in wanted if t not in TIERS]:
            print(f"unknown tiers: {unknown}", file=sys.stderr)
            return 2
        areas = {a for t in wanted for a in TIERS[t]}
        reasons, _ = read_open_rows(args.root / OPEN_ROWS)
        marks = {**evidence.propose(areas, reasons), **hand_written.propose(areas)}
        for rid in sorted(marks):
            print(f"{rid}\t" + " ".join(f"{c}={v}" for c, v in sorted(marks[rid].items())))
        print(f"{len(marks)} rows, {sum(map(len, marks.values()))} marks", file=sys.stderr)
        if args.why:
            with open(args.why, "w", encoding="utf8") as out:
                out.write("id\tarea\tkind\tcolumn\twhy\n")
                for r in rows:
                    if r["area"] not in areas:
                        continue
                    for c in CRATES:
                        if r.get(c) == "." and c not in marks.get(r["id"], {}):
                            why = "no fixture exercises it" if not cov.hits[r["id"]] else evidence.gap(c, r)
                            out.write(f"{r['id']}\t{r['area']}\t{r['kind']}\t{c}\t{why}\n")
        if args.apply:
            write_marks(checklist, marks)
        return 0
    failures, warnings = gates(rows, evidence, cov)
    failures += hand_written.gates()
    open_rows, open_problems = read_open_rows(args.root / OPEN_ROWS)
    failures += open_problems
    gated = {a for t in args.gate.split(",") if t for a in TIERS[t]}
    tracker = args.root / TRACKER
    tracker_text = tracker.read_text(encoding="utf8") if tracker.exists() else ""
    failures += status_gates(rows + hand_written.rows, evidence, hand_written, open_rows, gated, tracker_text,
                             retention_tests_exist(crates_dir) or not gated)
    for scan in scans.values():
        failures += [f"spec-audit owner names no element: {b}" for b in scan.bad_owners]
    names = {r["id"] for r in all_rows}
    failures += [f"spec-audit comment names no row: {n}" for n in sorted(evidence.model.noted - names)]
    areas = summary_table(rows, tiers, cov)
    samples = {c: len(s.refs) for c, s in scans.items()}
    print_report(rows, areas, tiers, cov, files, problems, failures, warnings, samples)

    if args.gaps:
        with open(args.gaps, "w", encoding="utf8") as out:
            out.write("id\tarea\tkind\timport\texport\tmodel\tfixture_hits\tfirst_fixture\n")
            for r in rows:
                rid = r["id"]
                out.write(f"{rid}\t{r['area']}\t{r['kind']}\t{tiers['I'].get(rid, 'none')}\t{tiers['E'].get(rid, 'none')}\t"
                          f"{tiers['M'].get(rid, 'none')}\t{cov.hits[rid]}\t{cov.first.get(rid, '')}\n")
    if args.json:
        args.json.write_text(json.dumps({
            "rows": len(rows),
            "areas": {a: dict(c) for a, c in areas.items()},
            "failures": failures + problems,
            "warnings": len(warnings),
            "refused": cov.refused,
            "unmatched": dict(cov.unmatched),
        }, indent=1, sort_keys=True), encoding="utf8")
    return 1 if args.check and (failures or problems) else 0


if __name__ == "__main__":
    sys.exit(main())
