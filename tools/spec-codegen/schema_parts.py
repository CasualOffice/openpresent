#!/usr/bin/env python3
"""T1K: the schemas of SmartArt, equations and ink, as a table of types.

Read by generate.py, which turns it into typed values, readers and writers, and by
fixtures/tools/generate_graphics.py, which turns it into decks that hold every row. So
what is written in the fixtures and what the code handles come from one reading.

Content models are reduced to what a reader and writer need. A child that may repeat
alone is a list of that child. A repeating choice, whose order means something, is one
ordered list of items. Anything else is an optional field. Groups are expanded in place.
"""
import pathlib
import re
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[2]
XS = "{http://www.w3.org/2001/XMLSchema}"

DML = "http://schemas.openxmlformats.org/drawingml/2006/main"
DGM = "http://schemas.openxmlformats.org/drawingml/2006/diagram"
DSP = "http://schemas.microsoft.com/office/drawing/2008/diagram"
DGM14 = "http://schemas.microsoft.com/office/drawing/2010/diagram"
DGM1611 = "http://schemas.microsoft.com/office/drawing/2016/11/diagram"
DGM1612 = "http://schemas.microsoft.com/office/drawing/2016/12/diagram"
MATH = "http://schemas.openxmlformats.org/officeDocument/2006/math"
INKML = "http://www.w3.org/2003/InkML"
MSINK = "http://schemas.microsoft.com/ink/2010/main"
IACT = "http://schemas.microsoft.com/office/powerpoint/2014/inkAction"
INK2016 = "http://schemas.microsoft.com/office/drawing/2016/ink"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
SHARED = "http://schemas.openxmlformats.org/officeDocument/2006/sharedTypes"
WML = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
XSD = "http://www.w3.org/2001/XMLSchema"
XML = "http://www.w3.org/XML/1998/namespace"

STRICT = {
    DGM: "http://purl.oclc.org/ooxml/drawingml/diagram",
    MATH: "http://purl.oclc.org/ooxml/officeDocument/math",
}

# module, source file, namespace, prefix, name prefix for Rust types
SOURCES = {
    "diagram": [
        ("schemas/ooxml/dml-diagram.xsd", DGM, "dgm", ""),
        ("schemas/ms-extensions/ms-odrawxml-06.xsd", DSP, "dsp", "Dsp"),
        ("schemas/ms-extensions/ms-odrawxml-16.xsd", DGM14, "dgm14", "Dgm14"),
        ("schemas/ms-extensions/ms-odrawxml-26.xsd", DGM1611, "dgm1611", "Dgm1611"),
        ("schemas/ms-extensions/ms-odrawxml-30.xsd", DGM1612, "dgm1612", "Dgm1612"),
    ],
    "math": [
        ("schemas/ooxml/shared-math.xsd", MATH, "m", ""),
    ],
    "ink": [
        ("tools/spec-codegen/inkml.xsd", INKML, "inkml", ""),
        ("schemas/ms-extensions/ms-odrawxml-07.xsd", MSINK, "msink", "Msink"),
        ("schemas/ms-extensions/ms-odrawxml-19.xsd", IACT, "iact", "Action"),
        ("schemas/ms-extensions/ms-odrawxml-28.xsd", INK2016, "inkeffects", "Ink2016"),
    ],
}

SCALARS = {
    "string": "String", "token": "String", "anyURI": "String", "ID": "String", "IDREF": "String",
    "NCName": "String", "QName": "String", "hexBinary": "String", "base64Binary": "String", "decimal": "String",
    "dateTime": "String", "language": "String", "normalizedString": "String", "id": "String",
    "boolean": "bool", "int": "i32", "integer": "i64", "long": "i64", "short": "i16",
    "unsignedInt": "u32", "unsignedLong": "u64", "unsignedShort": "u16", "unsignedByte": "u8", "byte": "i8",
    "nonNegativeInteger": "u64", "positiveInteger": "u64",
    "double": "Float64", "float": "Float32",
}


class Attr:
    def __init__(self, name, ns, st, required, rel=False):
        self.name, self.ns, self.st, self.required, self.rel = name, ns, st, required, rel
        self.local = name.split(":")[-1]
        self.qualified = False  # `attributeFormDefault="qualified"`: spelled with a prefix


class Child:
    """One element a type may hold."""

    def __init__(self, local, ns, typ, repeats, in_items, required):
        self.local, self.ns, self.type, self.repeats, self.in_items, self.required = local, ns, typ, repeats, in_items, required


class Type:
    def __init__(self, module, ns, prefix, tag, name):
        self.module, self.ns, self.prefix, self.tag, self.name = module, ns, prefix, tag, name
        self.attrs, self.children, self.groups = [], [], []
        self.text = None  # the simple type of simple content
        self.choices = []  # lists of locals of which a file may hold one
        self.retained_groups = []


class Simple:
    def __init__(self, module, ns, tag, name, node, resolve):
        self.module, self.ns, self.tag, self.name = module, ns, tag, name
        self.values = [e.get("value") for e in node.iter(f"{XS}enumeration")]
        self.union = None
        self.list = None
        self.base = None
        self.min = self.max = None
        self.pattern = None
        for u in node.findall(f"{XS}union"):
            self.union = [resolve(m) for m in (u.get("memberTypes") or "").split()]
            for inner in u.findall(f"{XS}simpleType"):
                raise SystemExit(f"{name}: an anonymous union member")
        for li in node.findall(f"{XS}list"):
            self.list = resolve(li.get("itemType"))
        for r in node.findall(f"{XS}restriction"):
            self.base = resolve(r.get("base"))
            for facet, slot in (("minInclusive", "min"), ("maxInclusive", "max")):
                f = r.find(f"{XS}{facet}")
                if f is not None:
                    setattr(self, slot, int(f.get("value")))
            f = r.find(f"{XS}pattern")
            if f is not None:
                self.pattern = f.get("value")


class Schemas:
    def __init__(self, root=ROOT):
        self.types, self.simples, self.elements, self.groups, self.attr_groups = {}, {}, {}, {}, {}
        self.attributes = {}
        self.attr_qualified = {}
        self.order = []
        self.prefix_of = {}
        self.tag_of = {}
        for module, sources in SOURCES.items():
            for path, ns, prefix, tag in sources:
                self.prefix_of[ns] = prefix
                self.tag_of[ns] = tag
                self._load(root / path, module, ns, prefix, tag)
        self.children_expanded = set()
        for key, t in self.types.items():
            self._expand(t)

    def _load(self, path, module, ns, prefix, tag):
        prefixes = {}
        for _, (p, uri) in ET.iterparse(path, events=("start-ns",)):
            prefixes.setdefault(p, uri)
        root = ET.parse(path).getroot()
        self.attr_qualified[ns] = root.get("attributeFormDefault") == "qualified"

        def resolve(q):
            if not q:
                return None
            p, _, local = q.rpartition(":")
            if not p:
                return (prefixes.get("", ns), local)
            return (prefixes.get(p, {"inkml": INKML, "xml": XML}.get(p, p)), local)

        for node in root:
            t, name = node.tag.replace(XS, ""), node.get("name")
            if not name:
                continue
            key = (ns, name)
            if t == "complexType":
                self.types[key] = Type(module, ns, prefix, tag, name)
                self.types[key].node = node
                self.types[key].resolve = resolve
                self.order.append(key)
            elif t == "simpleType":
                self.simples[key] = Simple(module, ns, tag, name, node, resolve)
            elif t == "element":
                self.elements[key] = resolve(node.get("type"))
            elif t == "group":
                self.groups[key] = (node, resolve)
            elif t == "attributeGroup":
                self.attr_groups[key] = (node, resolve)
            elif t == "attribute":
                self.attributes[key] = resolve(node.get("type"))

    def _expand(self, t):
        self._particles(t, t.node, t.resolve, False, False, False)
        names = {}
        kept = []
        for c in t.children:
            old = names.get((c.ns, c.local))
            if old is None:
                names[(c.ns, c.local)] = c
                kept.append(c)
            else:
                old.repeats = True
                old.in_items = old.in_items or c.in_items
        t.children = kept

    def _occurs(self, node):
        raw = node.get("maxOccurs", "1")
        return raw == "unbounded" or int(raw) > 1

    def _particles(self, t, node, resolve, repeated, optional, in_choice):
        for ch in node:
            tag = ch.tag.replace(XS, "")
            if tag in ("annotation",):
                continue
            if tag in ("sequence", "all"):
                many = repeated or self._occurs(ch)
                self._particles(t, ch, resolve, many, optional or ch.get("minOccurs") == "0", False)
            elif tag == "choice":
                many = repeated or self._occurs(ch)
                before = len(t.children)
                self._particles(t, ch, resolve, many, True, True)
                t.choices.append([c.local for c in t.children[before:]])
            elif tag == "group":
                target = resolve(ch.get("ref"))
                many = repeated or self._occurs(ch)
                opt = optional or ch.get("minOccurs") == "0"
                if target in self.groups:
                    group, group_resolve = self.groups[target]
                    self._particles(t, group, group_resolve, many, opt, in_choice)
                else:
                    t.retained_groups.append((target, many))
            elif tag in ("complexContent", "simpleContent"):
                self._particles(t, ch, resolve, repeated, optional, in_choice)
            elif tag == "extension":
                base = resolve(ch.get("base"))
                if base in self.types:
                    base_type = self.types[base]
                    self._particles(t, base_type.node, base_type.resolve, repeated, optional, in_choice)
                else:
                    t.text = base
                self._particles(t, ch, resolve, repeated, optional, in_choice)
            elif tag == "restriction":
                self._particles(t, ch, resolve, repeated, optional, in_choice)
            elif tag == "element":
                required = not optional and not in_choice and ch.get("minOccurs", "1") != "0"
                if ch.get("ref"):
                    q = resolve(ch.get("ref"))
                    t.children.append(Child(q[1], q[0], self.elements.get(q, q), self._occurs(ch), repeated, required))
                else:
                    t.children.append(Child(ch.get("name"), t.ns, resolve(ch.get("type")), self._occurs(ch), repeated, required))
            elif tag == "attribute":
                self._attribute(t, ch, resolve)
            elif tag == "attributeGroup":
                group, group_resolve = self.attr_groups.get(resolve(ch.get("ref")), (None, None))
                if group is not None:
                    self._particles(t, group, group_resolve, repeated, optional, in_choice)

    def _attribute(self, t, ch, resolve):
        required = ch.get("use") == "required"
        if ch.get("ref"):
            q = resolve(ch.get("ref"))
            if q[0] == REL:
                t.attrs.append(Attr(f"r:{q[1]}", REL, (XSD, "string"), required, rel=True))
            elif q[0] == XML:
                t.attrs.append(Attr(f"xml:{q[1]}", XML, (XSD, "string"), required))
            else:
                t.attrs.append(Attr(q[1], q[0], self.attributes.get(q, (XSD, "string")), required))
        else:
            attr = Attr(ch.get("name"), "", resolve(ch.get("type")) or (XSD, "string"), required)
            attr.qualified = self.attr_qualified.get(t.ns, False)
            t.attrs.append(attr)

    def module_types(self, module):
        return [self.types[k] for k in self.order if self.types[k].module == module]

    def module_simples(self, module):
        return [s for s in self.simples.values() if s.module == module]

    def module_roots(self, module):
        namespaces = {ns for _, ns, _, _ in SOURCES[module]}
        return [(k, v) for k, v in self.elements.items() if k[0] in namespaces]
