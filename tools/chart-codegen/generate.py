#!/usr/bin/env python3
"""T1J-01: the chart model, its reader and its writer, generated from the vendored schemas.

Chart markup is 1,764 checklist rows of one repeated shape, so one schema walk writes three
views of it, which cannot then disagree about a name, an order or a type:

  crates/casual-slide-model/src/chart/<ns>.rs    plain typed values, one struct per type
  crates/casual-slide-import/src/chart/<ns>.rs   reads a tree into those values
  crates/casual-slide-export/src/chart/<ns>.rs   says which elements those values stand for

Hand-written beside them, in each crate's src/chart.rs: what the schema cannot say (bounds,
reports, what a damaged part does, how a patch finds its nodes), and the few types the
generator is told to treat by hand (`HAND` below). What generated code does is tested by
crates/casual-slide-{import,export}/tests/chart.rs, row by row against docs/spec/checklist.tsv:
generating it is not evidence that it works.

DrawingML inside a chart is not generated. `a:CT_ShapeProperties`, `a:CT_TextBody` and the
colour choice use Tiers 1E, 1F and 1C's readers and writers; every other DrawingML type
(`cdr:` non-visual properties, `cs:` text properties) is `Retained`: named in the model, kept
in the tree, reported, and not editable.

Formatting is the generator's own: the modules are `#[rustfmt::skip]`, so this tool needs no
rustfmt and its output is the same everywhere.

    python3 tools/chart-codegen/generate.py          # write
    python3 tools/chart-codegen/generate.py --check  # fail if anything on disk differs
"""
import argparse
import pathlib
import re
import sys
import xml.etree.ElementTree as ET
from collections import OrderedDict

XS = "{http://www.w3.org/2001/XMLSchema}"
ROOT = pathlib.Path(__file__).resolve().parents[2]
DML = "http://schemas.openxmlformats.org/drawingml/2006/main"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
SHARED = "http://schemas.openxmlformats.org/officeDocument/2006/sharedTypes"
A14 = "http://schemas.microsoft.com/office/drawing/2010/main"
XSD = "http://www.w3.org/2001/XMLSchema"

# module, schema file, row-id key, namespace uris (Transitional, then Strict), prefix
MODULES = [
    ("c", "schemas/ooxml/dml-chart.xsd", "c",
     ["http://schemas.openxmlformats.org/drawingml/2006/chart", "http://purl.oclc.org/ooxml/drawingml/chart"]),
    ("cdr", "schemas/ooxml/dml-chartDrawing.xsd", "cdr",
     ["http://schemas.openxmlformats.org/drawingml/2006/chartDrawing", "http://purl.oclc.org/ooxml/drawingml/chartDrawing"]),
    ("c14", "schemas/ms-extensions/ms-odrawxml-10.xsd", "x:drawing/2007/8/2/chart",
     ["http://schemas.microsoft.com/office/drawing/2007/8/2/chart"]),
    ("cdr14", "schemas/ms-extensions/ms-odrawxml-08.xsd", "x:drawing/2010/chartDrawing",
     ["http://schemas.microsoft.com/office/drawing/2010/chartDrawing"]),
    ("c15", "schemas/ms-extensions/ms-odrawxml-13.xsd", "x:drawing/2012/chart",
     ["http://schemas.microsoft.com/office/drawing/2012/chart"]),
    ("cs", "schemas/ms-extensions/ms-odrawxml-15.xsd", "x:drawing/2012/chartStyle",
     ["http://schemas.microsoft.com/office/drawing/2012/chartStyle"]),
    ("c16", "schemas/ms-extensions/ms-odrawxml-20.xsd", "x:drawing/2014/chart",
     ["http://schemas.microsoft.com/office/drawing/2014/chart"]),
    ("c16ac", "schemas/ms-extensions/ms-odrawxml-21.xsd", "x:drawing/2014/chart/ac",
     ["http://schemas.microsoft.com/office/drawing/2014/chart/ac"]),
    ("cx", "schemas/ms-extensions/ms-odrawxml-22.xsd", "x:drawing/2014/chartex",
     ["http://schemas.microsoft.com/office/drawing/2014/chartex"]),
    ("c16r3", "schemas/ms-extensions/ms-odrawxml-31.xsd", "x:drawing/2017/03/chart",
     ["http://schemas.microsoft.com/office/drawing/2017/03/chart"]),
    ("c16r2", "schemas/ms-extensions/ms-odrawxml-42.xsd", "x:drawing/2015/06/chart",
     ["http://schemas.microsoft.com/office/drawing/2015/06/chart"]),
]
MODULE_OF_URI = {m[3][0]: m[0] for m in MODULES}
URIS_OF = {m[0]: m[3] for m in MODULES}
KEY_OF = {m[0]: m[2] for m in MODULES}
# Globals that are a part's root, not something an extension holds.
ROOTS = {"c": {"chartSpace", "userShapes", "chart"}, "cx": {"chartSpace", "chart"},
         "cs": {"colorStyle", "chartStyle"}, "cdr14": {"contentPart"}}
EXT_MODULES = ("c14", "c15", "c16", "c16ac", "c16r2", "c16r3")

BUILTIN = {
    "boolean": "bool", "double": "Float64", "unsignedInt": "u32", "int": "i32", "integer": "i64",
    "byte": "i8", "unsignedByte": "u8", "unsignedShort": "u16", "string": "String", "token": "String",
    "language": "String", "base64Binary": "String",
}
# A list a limit in ChartLimits counts, by element.
BOUNDED = {
    "pt": "Points", "ser": "Series", "dPt": "DataPoints", "dLbl": "DataLabels", "lvl": "Levels",
    "relSizeAnchor": "Anchors", "absSizeAnchor": "Anchors", "ext": "Extensions", "series": "Series",
    "dataLabel": "DataLabels", "dataPoint": "DataPoints",
}
# Written by hand in the crates' src/chart.rs: a signed axis id (T1J-04).
AXIS_ID_ELEMENTS = {"axId", "crossAx"}
RUST_WORDS = {"as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let",
              "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
              "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final",
              "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"}


def snake(name):
    s = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name.replace(".", "_").replace("-", "_"))
    s = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", "_", s).lower()
    return s + "_" if s in RUST_WORDS else s


def camel(name):
    parts = re.split(r"[^A-Za-z0-9]+", name)
    out = "".join(p[:1].upper() + p[1:] for p in parts if p)
    if not out:
        out = "Empty"
    if out[0].isdigit():
        out = "V" + out
    return out


def type_name(name):
    return camel(re.sub(r"^(CT|EG|AG)_", "", name))


def enum_name(name):
    return camel(re.sub(r"^ST_", "", name)) + "Value"


class Mod:
    def __init__(self, name, path, key, uris):
        self.name, self.key, self.uris = name, key, uris
        self.prefixes = {}
        for _, (pfx, uri) in ET.iterparse(path, events=("start-ns",)):
            self.prefixes[pfx] = uri
        self.root = ET.parse(path).getroot()
        self.qualified = self.root.get("elementFormDefault") == "qualified"
        self.attr_qualified = self.root.get("attributeFormDefault") == "qualified"
        self.complex, self.groups, self.simple, self.elements, self.attrs = OrderedDict(), {}, OrderedDict(), OrderedDict(), OrderedDict()
        for node in self.root:
            tag, name = node.tag.replace(XS, ""), node.get("name")
            if tag == "complexType":
                self.complex[name] = node
            elif tag == "group":
                self.groups[name] = node
            elif tag == "simpleType":
                self.simple[name] = node
            elif tag == "element":
                self.elements[name] = node
            elif tag == "attribute":
                self.attrs[name] = node

    def q(self, text):
        """(uri, local) of a QName as this schema spells it."""
        pfx, _, local = text.rpartition(":")
        return (self.prefixes.get(pfx, ""), local)


class Schemas:
    def __init__(self):
        self.mods = OrderedDict((m[0], Mod(m[0], ROOT / m[1], m[2], m[3])) for m in MODULES)
        self.dml = ET.parse(ROOT / "schemas/ooxml/dml-main.xsd").getroot()

    def mod_of(self, uri):
        return self.mods.get(MODULE_OF_URI.get(uri))

    def dml_group_names(self, name):
        for node in self.dml:
            if node.tag == XS + "group" and node.get("name") == name:
                return [e.get("name") for e in node.iter(XS + "element")]
        raise SystemExit(f"no group a:{name}")


class Simple:
    """How a simple type is held. `rust` names the type; `allowed` is a closure, or None."""

    def __init__(self, rust, allowed=None, enums=()):
        self.rust, self.allowed, self.enums = rust, allowed, tuple(enums)


def num_literal(base, value):
    if base == "Float64":
        return repr(float(value))
    return str(int(float(value)))


class Walker:
    def __init__(self, schemas):
        self.s = schemas
        self.types = OrderedDict()  # (mod, name) -> Type
        self.memo_simple = {}

    # ------------------------------------------------------------ simple types

    def simple(self, uri, local):
        key = (uri, local)
        if key not in self.memo_simple:
            self.memo_simple[key] = self._simple(uri, local)
        return self.memo_simple[key]

    def _simple(self, uri, local):
        if uri == XSD:
            return Simple(BUILTIN[local])
        mod = self.s.mod_of(uri)
        if mod is None or local not in mod.simple:
            if mod is not None and local in mod.complex:
                raise SystemExit(f"{local}: complex used as simple")
            return Simple("String")  # a type of another schema's, kept as written
        return self.simple_node(mod, local, mod.simple[local])

    def simple_node(self, mod, local, node):
        restriction = node.find(XS + "restriction")
        union = node.find(XS + "union")
        lst = node.find(XS + "list")
        if restriction is not None:
            values = [e.get("value") for e in restriction.findall(XS + "enumeration")]
            if values:
                return Simple(f"super::{mod.name}::{enum_name(local)}", enums=[(mod.name, local)])
            base = self.simple(*mod.q(restriction.get("base")))
            bounds = {c.tag.replace(XS, ""): c.get("value") for c in restriction}
            lo, hi = bounds.get("minInclusive"), bounds.get("maxInclusive")
            exclusive = bounds.get("minExclusive")
            if base.rust == "Float64" and (lo or hi or exclusive):
                checks = []
                if lo is not None:
                    checks.append(f"v.get() >= {num_literal('Float64', lo)}")
                if exclusive is not None:
                    checks.append(f"v.get() > {num_literal('Float64', exclusive)}")
                if hi is not None:
                    checks.append(f"v.get() <= {num_literal('Float64', hi)}")
                return Simple("Float64", "|v: &Float64| " + " && ".join(checks))
            if base.rust != "String" and (lo is not None or hi is not None or exclusive is not None):
                checks = []
                if exclusive is not None:
                    checks.append(f"*v > {num_literal(base.rust, exclusive)}")
                if lo is not None and int(float(lo)) != 0 or lo is not None and base.rust.startswith("i"):
                    checks.append(f"*v >= {num_literal(base.rust, lo)}")
                if hi is not None:
                    checks.append(f"*v <= {num_literal(base.rust, hi)}")
                if not checks:
                    return Simple(base.rust)
                return Simple(base.rust, f"|v: &{base.rust}| " + " && ".join(checks))
            return Simple(base.rust, base.allowed, base.enums)
        if lst is not None:
            item = self.simple(*mod.q(lst.get("itemType")))
            if item.rust.startswith("Open<"):
                return Simple(item.rust.replace("Open<", "OpenList<", 1), enums=item.enums)
            return Simple("String")
        if union is not None:
            names = (union.get("memberTypes") or "").split()
            members = [mod.q(n) for n in names]
            inner = union.findall(XS + "simpleType")
            described = [self.simple(*m) for m in members]
            described += [self.simple_node(mod, local, n) for n in inner]
            enums = [d for d in described if d.enums]
            open_string = any(d.rust == "String" and not d.allowed for d in described)
            if len(enums) == 1 and open_string:
                path = enums[0].rust
                return Simple(f"Open<{path}>", enums=enums[0].enums)
            # A percent spelling beside a numeric range: both name one range.
            percent = []
            for (uri, name) in members:
                m = self.s.mod_of(uri)
                if m is not None and name in m.simple:
                    n = m.simple[name].find(XS + "restriction")
                    if n is not None:
                        pattern = n.find(XS + "pattern")
                        if pattern is not None and pattern.get("value", "").endswith("%"):
                            percent.append(name)
            numeric = [d for d in described if d.allowed and d.rust != "String"]
            if percent and len(numeric) == 1:
                # The range is in the numeric member's closure, as `*v >= a && *v <= b`.
                closure = numeric[0].allowed
                lo = re.search(r"\*v >= (-?\d+)", closure)
                hi = re.search(r"\*v <= (-?\d+)", closure)
                lo = lo.group(1) if lo else "0"
                hi = hi.group(1) if hi else "i64::MAX"
                return Simple("String", f"|v: &String| percent_or_number(v, {lo}, {hi})", [e for d in enums for e in d.enums])
            return Simple("String", enums=[e for d in enums for e in d.enums])
        raise SystemExit(f"{local}: unhandled simple type")


class Field:
    """One attribute, element, list, text content or repeated choice of a type."""

    def __init__(self, kind, local, rust, doc):
        self.kind, self.local, self.rust, self.doc = kind, local, rust, doc
        self.many = False
        self.required = False
        self.payload = None  # ("type", mod, name) | ("simple", Simple) | ("shape",) | ("text",) | ("color",) | ("retained",) | ("axis",) | ("items",)
        self.space = None    # module of the element
        self.names = None    # for an element standing for several names
        self.variants = None  # for a repeated choice: list of Field
        self.enum = None
        self.qualified = None
        self.note = None
        self.host_doc = None


class Type:
    def __init__(self, mod, name, fields, groups):
        self.mod, self.name, self.fields, self.groups = mod, name, fields, groups
        self.rust = type_name(name)
        self.order = []
        self.hosts = set()
        self.recursive = False
        self.choices = []


def occurs_many(node):
    m = node.get("maxOccurs", "1")
    return m == "unbounded" or int(m) > 1


def is_optional(node):
    return node.get("minOccurs", "1") == "0"


class Builder:
    def __init__(self, schemas):
        self.s = schemas
        self.w = Walker(schemas)
        self.types = OrderedDict()
        self.dml_notes = set()
        for mod in schemas.mods.values():
            for name, node in mod.complex.items():
                self.types[(mod.name, name)] = self.complex(mod, name, node)
        self.link()

    # ------------------------------------------------------------ content

    def element_field(self, mod, node, many, required):
        ref = node.get("ref")
        if ref:
            uri, local = mod.q(ref)
            target = self.s.mod_of(uri)
            if uri == DML:
                f = Field("element", local, snake(local), f"`{local}`")
                f.space = "dml"
                f.payload = ("retained",)
            elif target is not None:
                decl = target.elements[local]
                f = self.typed(target, local, target.q(decl.get("type")), target.name)
            else:
                raise SystemExit(f"unhandled ref {ref}")
        else:
            local = node.get("name")
            inner = node.find(XS + "complexType")
            if inner is not None:
                raise SystemExit(f"anonymous type under {local}")
            f = self.typed(mod, local, mod.q(node.get("type")), mod.name)
        f.many = many or occurs_many(node)
        f.required = required and not is_optional(node)
        return f

    def typed(self, mod, local, qtype, space):
        uri, tname = qtype
        f = Field("element", local, snake(local), f"`{local}`")
        f.space = space
        if uri == DML:
            if tname == "CT_ShapeProperties":
                f.payload = ("shape",)
            elif tname == "CT_TextBody":
                f.payload = ("text",)
            else:
                f.payload = ("retained",)
        elif uri == A14:
            f.payload = ("retained",)
        else:
            target = self.s.mod_of(uri)
            if target is not None and tname in target.complex:
                if tname == "CT_UnsignedInt" and local in AXIS_ID_ELEMENTS:
                    f.payload = ("axis",)
                else:
                    f.payload = ("type", target.name, tname)
            else:
                f.payload = ("simple", self.w.simple(uri, tname))
        return f

    def particles(self, mod, node, many, required, groups):
        """Fields of a particle, in order. A repeated choice stays one field: its order is
        content, as it is the order charts draw in."""
        out = []
        tag = node.tag.replace(XS, "")
        if tag == "element":
            return [self.element_field(mod, node, many, required)]
        if tag == "sequence":
            inner_many = many or occurs_many(node)
            inner_required = required and not is_optional(node)
            for ch in node:
                out += self.particles(mod, ch, inner_many, inner_required, groups)
            return out
        if tag == "choice":
            for_each = many or occurs_many(node)
            for ch in node:
                out += self.particles(mod, ch, False, False, groups)
            if for_each:
                flat = []
                for f in out:
                    flat += f.variants if f.kind == "choice" else [f]
                return [self.choice_field(flat)]
            return out
        if tag == "group":
            uri, name = mod.q(node.get("ref"))
            if uri == DML:
                return [self.dml_group(name, many or occurs_many(node), required and not is_optional(node), mod, groups)]
            target = self.s.mod_of(uri)
            groups.append((target.name, name))
            gnode = target.groups[name]
            for ch in gnode:
                sub = self.particles(target, ch, many or occurs_many(node), required and not is_optional(node), groups)
                for f in sub:
                    if name.startswith("Group_"):
                        f.host_doc = name
                out += sub
            return out
        if tag == "any":
            f = Field("items", "#any", "items", "what an extension holds")
            f.payload = ("items",)
            f.many = True
            return [f]
        if tag in ("annotation",):
            return []
        raise SystemExit(f"unhandled particle {tag}")

    def dml_group(self, name, many, required, mod, groups):
        groups.append(("a", name))
        if name == "EG_ColorChoice":
            f = Field("element", "#color", "colors" if many else "color", "a colour, `a:EG_ColorChoice`")
            f.payload = ("color",)
        else:
            f = Field("element", "#transform", "transforms", "a colour transform, `a:EG_ColorTransform`, kept in the tree")
            f.payload = ("retained",)
            many = True
        f.names = self.s.dml_group_names(name)
        f.space = "dml"
        f.many = many
        f.required = required
        return f

    def choice_field(self, flat):
        f = Field("choice", "#choice", "choice", "the repeated choice, in the order the file has it")
        f.variants = flat
        f.many = True
        return f

    def complex(self, mod, name, node):
        groups = []
        fields = []
        attrs = []

        def attributes(holder):
            for ch in holder:
                tag = ch.tag.replace(XS, "")
                if tag == "attribute":
                    attrs.append(self.attribute(mod, ch))
                elif tag == "anyAttribute":
                    raise SystemExit(f"{name}: anyAttribute")

        simple_content = node.find(XS + "simpleContent")
        if simple_content is not None:
            ext = simple_content.find(XS + "extension")
            f = Field("value", "#text", "value", "the element's text")
            f.payload = ("simple", self.w.simple(*mod.q(ext.get("base"))))
            fields.append(f)
            attributes(ext)
        else:
            for ch in node:
                tag = ch.tag.replace(XS, "")
                if tag in ("sequence", "choice", "group", "element", "any"):
                    fields += self.particles(mod, ch, False, True, groups)
                elif tag == "attribute":
                    pass
                elif tag == "annotation":
                    pass
                else:
                    raise SystemExit(f"{name}: {tag}")
            attributes(node)
        # The same element twice in a sequence is one list: the second is one the schema's
        # choice between shapes of content reaches by another path.
        merged, seen = [], {}
        for f in fields:
            if f.kind == "element" and f.local in seen:
                seen[f.local].many = seen[f.local].many or f.many
                seen[f.local].required = seen[f.local].required and f.required
                continue
            if f.kind == "element":
                seen[f.local] = f
            merged.append(f)
        names = {f.rust for f in merged}
        for a in attrs:
            if a.rust in names:
                a.rust = a.rust.rstrip("_") + "_attr"
            names.add(a.rust)
        t = Type(mod.name, name, attrs + merged, groups)
        for f in merged:
            if f.kind == "element":
                t.order += f.names or [f.local]
            elif f.kind == "choice":
                for v in f.variants:
                    t.order += v.names or [v.local]
                t.choices.append(f)
                hint = CHOICE_NAMES.get((mod.name, name), ())
                at = len(t.choices) - 1
                f.rust = hint[at] if at < len(hint) else ("choice" if at == 0 else f"choice_{at + 1}")
                f.enum = t.rust + camel(f.rust)
                seen_variant = set()
                for v in f.variants:
                    v.variant = camel(v.local.lstrip("#"))
                    if v.variant in seen_variant:
                        raise SystemExit(f"{name}: repeated choice branch {v.local}")
                    seen_variant.add(v.variant)
        return t

    def attribute(self, mod, node):
        ref = node.get("ref")
        if ref:
            uri, local = mod.q(ref)
            if uri != REL:
                raise SystemExit(f"attribute ref {ref}")
            f = Field("attribute", local, snake(local), f"`{local}`")
            f.payload = ("rel",)
            f.required = node.get("use") == "required"
            return f
        local = node.get("name")
        f = Field("attribute", local, snake(local), f"`{local}`")
        typ = node.get("type")
        if typ:
            f.payload = ("simple", self.w.simple(*mod.q(typ)))
        else:
            inner = node.find(XS + "simpleType")
            f.payload = ("simple", self.w.simple_node(mod, local, inner) if inner is not None else Simple("String"))
        f.required = node.get("use") == "required"
        if mod.attr_qualified:
            f.qualified = mod.name
        return f

    # ------------------------------------------------------------ hosts and recursion

    def link(self):
        def fields_of(t):
            for f in t.fields:
                if f.kind == "choice":
                    yield from f.variants
                else:
                    yield f

        for t in self.types.values():
            for f in fields_of(t):
                if f.payload and f.payload[0] == "type":
                    self.types[(f.payload[1], f.payload[2])].hosts.add(f.local)
                if f.payload and f.payload[0] == "axis":
                    self.types[("c", "CT_UnsignedInt")].hosts.add(f.local)
        for mod in self.s.mods.values():
            for name, node in mod.elements.items():
                typ = node.get("type")
                if typ:
                    uri, tname = mod.q(typ)
                    target = self.s.mod_of(uri)
                    if target is not None and (target.name, tname) in self.types:
                        self.types[(target.name, tname)].hosts.add(name)
        edges = {k: {(f.payload[1], f.payload[2]) for f in fields_of(t) if f.payload and f.payload[0] == "type"}
                 for k, t in self.types.items()}
        for k, t in self.types.items():
            todo, seen = list(edges[k]), set()
            while todo:
                cur = todo.pop()
                if cur in seen:
                    continue
                seen.add(cur)
                todo += edges[cur]
            t.recursive = k in seen


# ---------------------------------------------------------------- emit

HEADER = "// Generated by tools/chart-codegen/generate.py from {schema}. Do not edit.\n"


def rust_path(from_mod, mod, rust):
    return rust if from_mod == mod else f"super::{mod}::{rust}"


def local_path(path, mod):
    return path.replace(f"super::{mod}::", "")


def field_type(f, mod):
    p = f.payload
    if p[0] == "type":
        return rust_path(mod, p[1], type_name(p[2]))
    if p[0] == "simple":
        return local_path(p[1].rust, mod)
    return {"shape": "ShapeProperties", "text": "TextBody", "color": "Color", "retained": "Retained", "axis": "AxisId", "rel": "String",
            "items": "ExtItem"}[p[0]]


def model_type(f, mod):
    inner = field_type(f, mod)
    boxed = f.payload[0] in ("type", "shape", "text")
    if f.many:
        return f"Vec<{inner}>"
    return f"Option<Box<{inner}>>" if boxed else f"Option<{inner}>"


def type_doc(t, s):
    names = [f"`{t.mod}:{t.name}`"]
    seen = set()
    for (gm, g) in t.groups:
        if (gm, g) not in seen:
            seen.add((gm, g))
            if g.startswith(("EG_", "AG_")) and gm != "a":
                names.append(f"`{gm}:{g}`")
    return ", with ".join([names[0], ", ".join(names[1:])]) if len(names) > 1 else names[0]


def field_docs(f, t):
    doc = f.doc
    if f.host_doc and t.hosts and f.kind == "element":
        # A `Group_*` has no doc comment of its own to name it: say it by an element holding it.
        doc += f", as in `{sorted(t.hosts)[0]}/{f.local}`"
    return doc + "."


def emit_model(builder, mod):
    m = builder.s.mods[mod]
    out = [HEADER.format(schema=pathlib.Path(dict((x[0], x[1]) for x in MODULES)[mod]).name)]
    out.append("//! Plain typed values for this namespace's types. See `tools/chart-codegen/generate.py`.\n\n")
    out.append("#[allow(unused_imports)] // Each namespace uses its own few of these.\n")
    out.append("use super::{AxisId, ExtItem, Open, OpenList, Retained};\n")
    out.append("#[allow(unused_imports)]\nuse crate::color::Color;\n#[allow(unused_imports)]\nuse crate::drawing::ShapeProperties;\n")
    out.append("#[allow(unused_imports)]\nuse crate::properties::{Float64, token_enum};\n#[allow(unused_imports)]\nuse crate::text::TextBody;\n\n")
    for name, node in m.simple.items():
        values = [e.get("value") for e in node.iter(XS + "enumeration")]
        if not values or node.find(XS + "restriction") is None:
            continue
        # `c:ST_MarkerStyle` is this type's superset. The audit knows an enum by its type's bare
        # name, so this one does not spell it, and the rows are held to the larger.
        doc = "`cs:` chart style marker styles" if (mod, name) == ("cs", "ST_MarkerStyle") else f"`{mod}:{name}`"
        out.append(f"token_enum!(\n    /// {doc}.\n    {enum_name(name)} {{\n")
        used = set()
        for v in values:
            var = camel(v)
            while var in used:
                var += "X"
            used.add(var)
            out.append(f'        {var} = "{v}",\n')
        out.append("    }\n);\n\n")
    for (tm, name), t in builder.types.items():
        if tm != mod:
            continue
        for c in t.choices:
            out.append(f"/// `{mod}:{name}`: one of its repeated choice.\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum {c.enum} {{\n")
            for v in c.variants:
                inner = field_type(v, mod)
                boxed = v.payload[0] in ("type", "shape", "text")
                out.append(f"    /// `{v.local}`.\n    {v.variant}({'Box<' + inner + '>' if boxed else inner}),\n")
            out.append("}\n\n")
        out.append(f"/// {type_doc(t, builder.s)}.\n#[derive(Debug, Clone, Default, PartialEq, Eq)]\npub struct {t.rust} {{\n")
        for f in t.fields:
            if f.kind == "choice":
                out.append(f"    /// {f.doc}.\n    pub {f.rust}: Vec<{f.enum}>,\n")
            elif f.kind in ("value",):
                out.append(f"    /// {f.doc}.\n    pub value: Option<{field_type(f, mod)}>,\n")
            elif f.kind == "items":
                out.append(f"    /// {f.doc}.\n    pub items: Vec<ExtItem>,\n")
            else:
                out.append(f"    /// {field_docs(f, t)}\n    pub {f.rust}: {model_type(f, mod)},\n")
        if (mod, name) in HAND_FIELDS:
            for (rust, typ, doc) in HAND_FIELDS[(mod, name)]:
                out.append(f"    /// {doc}\n    pub {rust}: {typ},\n")
        out.append("}\n\n")
    return "".join(out)


# Types nothing generated calls: their rows are in docs/spec, and the schemas give them no
# element, except `cx:CT_RelId`, which a graphic frame's `cx:chart` holds and `read_chart_reference` reads.
UNREACHED = {("c14", "CT_BooleanTrue"), ("c16", "CT_NumFilteredLiteralCache"), ("c16", "CT_StrFilteredLiteralCache"),
             ("c16", "CT_MultiLvlStrFilteredLiteralCache"), ("c16", "CT_LiteralDataChart"), ("cx", "CT_FeatureExtension"),
             ("cx", "CT_RelId"), ("cx", "CT_GeoHierarchyEntity")}

# Checks run once a type is read, each in the import crate's src/chart.rs.
HOOKS = {
    ("c", "CT_NumData"): "self.check_points(el, out.pt_count.as_ref().and_then(|c| c.val), out.pt.iter().map(|p| p.idx));",
    ("c", "CT_StrData"): "self.check_points(el, out.pt_count.as_ref().and_then(|c| c.val), out.pt.iter().map(|p| p.idx));",
    ("c", "CT_PlotArea"): "self.check_plot_area(el, &out);",
}

# What a type's repeated choices hold, where a number would say nothing.
CHOICE_NAMES = {("c", "CT_PlotArea"): ("charts", "axes")}

# Fields the schema cannot give, each handled in the crates' src/chart.rs.
HAND_FIELDS = {
    ("c", "CT_ChartSpace"): [("alternate_style", "Option<super::AlternateStyle>",
                              "`mc:AlternateContent` holding `c14:style`, with `c:style` as its fallback, as PowerPoint writes it.")],
    ("c", "CT_NumVal"): [("formatcode2", "Option<String>", "`c16r2:formatcode2` on a point, kept where Ignorable says it may be.")],
    ("cdr", "CT_RelSizeAnchor"): [("alternate_content_part", "Vec<super::AlternateContentPart>", "`mc:AlternateContent` holding `cdr14:contentPart`.")],
    ("cdr", "CT_AbsSizeAnchor"): [("alternate_content_part", "Vec<super::AlternateContentPart>", "`mc:AlternateContent` holding `cdr14:contentPart`.")],
}


def read_arm(f, mod, target, indent):
    """Code that reads one child into `target`."""
    p = f.payload
    pad = " " * indent
    if p[0] == "type":
        call = f"self.{p[1]}_{snake(p[2][3:] if p[2].startswith('CT_') else p[2]).rstrip('_')}(&child)?"
        value = f"Box::new({call})"
        if f.many:
            return f"{target}.push({call});"
        return f"{target} = Some({value});"
    if p[0] == "axis":
        return f"{target}.push(self.axis_id(&child)?);" if f.many else f"{target} = Some(self.axis_id(&child)?);"
    if p[0] == "shape":
        return f"{target}.push(*self.shape(&child)?);" if f.many else f"{target} = Some(self.shape(&child)?);"
    if p[0] == "text":
        return f"{target}.push(*self.text_body(&child)?);" if f.many else f"{target} = Some(self.text_body(&child)?);"
    if p[0] == "color":
        return f"{target}.extend(self.color_value(&child));" if f.many else f"{target} = self.color_value(&child);"
    if p[0] == "retained":
        return f"{target}.push(self.retain(&child));" if f.many else f"{target} = Some(self.retain(&child));"
    if p[0] == "simple":
        rust = local_path(p[1].rust, mod)
        if rust == "String":
            read = "self.r.text(&child)?"
        else:
            read = f"self.r.typed_text::<{rust}>(&child)?"
        if f.many:
            return f"{target}.extend({read});"
        return f"{target} = {read};"
    raise SystemExit(f"read_arm {p}")


def pattern(f):
    names = f.names or [f.local]
    return " | ".join(f'"{n}"' for n in names)


def guard_ns(f):
    return "ns::DML" if f.space == "dml" else "NS_" + (f.space or "").upper()


def single_is_some(f, target):
    return f"{target}.is_some()"


def emit_read_fn(t, builder, mod):
    fn = fn_name(t)
    lines = []
    owners = ",".join(sorted(t.hosts))
    if owners:
        lines.append(f"    // spec-audit: owner={owners}\n")
    inner = fn + "_inner" if t.recursive else fn
    if t.recursive:
        lines.append(f"    pub(crate) fn {fn}(&mut self, el: &Elem) -> Res<{t.rust}> {{\n        self.enter()?;\n"
                     f"        let out = self.{inner}(el);\n        self.leave();\n        out\n    }}\n\n")
        if owners:
            lines.append(f"    // spec-audit: owner={owners}\n")
    if (t.mod, t.name) in UNREACHED:
        lines.append("    #[allow(dead_code)] // No element in these schemas holds this type, or a hand-written reader reads it.\n")
    lines.append(f"    pub(crate) fn {inner}(&mut self, el: &Elem) -> Res<{t.rust}> {{\n")
    has_attr = any(f.kind == "attribute" for f in t.fields) or (t.mod, t.name) == ("c", "CT_NumVal")
    lines.append(f"        let {'mut ' if has_attr else ''}e = el.clone();\n")
    empty = not t.fields and (t.mod, t.name) not in HAND_FIELDS
    lines.append(f"        let {'' if empty else 'mut '}out = {t.rust}::default();\n")
    for f in t.fields:
        if f.kind != "attribute":
            continue
        p = f.payload
        if p[0] == "rel":
            lines.append(f'        out.{f.rust} = self.rel(&mut e, "{f.local}", {"true" if f.required else "false"})?;\n')
            continue
        rust = local_path(p[1].rust, mod)
        closure = p[1].allowed or f"|_: &{rust}| true"
        if f.qualified:
            uris = "NS_" + f.qualified.upper()
            lines.append(f'        out.{f.rust} = self.qualified_attr::<{rust}>(&mut e, {uris}, "{f.local}", {"true" if f.required else "false"})?;\n')
        elif f.required:
            lines.append(f'        out.{f.rust} = self.r.required(&mut e, "{f.local}", {closure})?;\n')
        elif p[1].allowed:
            lines.append(f'        out.{f.rust} = self.r.attr_in(&mut e, "{f.local}", {closure})?;\n')
        else:
            lines.append(f'        out.{f.rust} = self.r.attr::<{rust}>(&mut e, "{f.local}")?;\n')
    if (t.mod, t.name) == ("c", "CT_NumVal"):
        lines.append('        // spec-audit: x:drawing/2015/06/chart:@formatcode2\n')
        lines.append('        out.formatcode2 = self.qualified_attr::<String>(&mut e, NS_C16R2, "formatcode2", false)?;\n')
    lines.append("        self.r.finish(&e);\n")
    value = next((f for f in t.fields if f.kind == "value"), None)
    if value:
        rust = local_path(value.payload[1].rust, mod)
        lines.append(("        out.value = self.r.text(el)?;\n" if rust == "String" else
                      f"        out.value = self.r.typed_text::<{rust}>(el)?;\n"))
        lines.append("        let _ = &e;\n        Ok(out)\n    }\n\n")
        return "".join(lines)
    items = next((f for f in t.fields if f.kind == "items"), None)
    if items is not None:
        for row in any_rows(t):
            lines.append(f"        // spec-audit: {row}\n")
    lines.append("        for child in self.r.children(el)? {\n")
    if items is not None:
        lines.append("            let _ = &child;\n            out.items.push(self.ext_item(&child)?);\n        }\n")
    else:
        lines.append("            match child.local.as_str() {\n")
        for f in t.fields:
            if f.kind == "element":
                bound = BOUNDED.get(f.local)
                body = read_arm(f, mod, f"out.{f.rust}", 20)
                lines.append(f"                {pattern(f)} if child.in_namespace({guard_ns(f)}) => {{\n")
                if f.many:
                    if bound:
                        lines.append(f"                    self.charge(ChartBound::{bound})?;\n")
                    lines.append(f"                    {body}\n")
                elif f.payload[0] == "color":
                    lines.append(f"                    if out.{f.rust}.is_some() {{\n                        self.dup(&child);\n                    }} else {{\n                        {body}\n                    }}\n")
                else:
                    lines.append(f"                    if out.{f.rust}.is_some() {{\n                        self.dup(&child);\n                    }} else {{\n                        {body}\n                    }}\n")
                lines.append("                }\n")
            elif f.kind == "choice":
                for v in f.variants:
                    bound = BOUNDED.get(v.local)
                    lines.append(f"                {pattern(v)} if child.in_namespace({guard_ns(v)}) => {{\n")
                    if bound:
                        lines.append(f"                    self.charge(ChartBound::{bound})?;\n")
                    lines.append(f"                    out.{f.rust}.push({f.enum}::{v.variant}({choice_value(v, mod)}));\n                }}\n")
        if (t.mod, t.name) == ("c", "CT_ChartSpace"):
            lines.append('                "AlternateContent" if child.in_namespace(NS_MC) && out.alternate_style.is_none() => {\n'
                         "                    out.alternate_style = self.alternate_style(&child)?;\n                }\n")
        if (t.mod, t.name) in (("cdr", "CT_RelSizeAnchor"), ("cdr", "CT_AbsSizeAnchor")):
            lines.append('                "AlternateContent" if child.in_namespace(NS_MC) => {\n'
                         "                    out.alternate_content_part.extend(self.alternate_content_part(&child)?);\n                }\n")
        lines.append("                _ => self.other(&child),\n            }\n        }\n")
    for f in t.fields:
        if f.kind == "element" and f.required:
            present = f"out.{f.rust}.is_empty()" if f.many else f"out.{f.rust}.is_none()"
            local = "scrgbClr" if f.names and f.payload[0] == "color" else f.local.lstrip("#")
            lines.append(f'        if {present} {{\n            self.r.note(K::MissingRequired, el, Some("{local}"));\n        }}\n')
    if (t.mod, t.name) in HOOKS:
        lines.append(f"        {HOOKS[(t.mod, t.name)]}\n")
    lines.append("        Ok(out)\n    }\n\n")
    return "".join(lines)


def choice_value(v, mod):
    p = v.payload
    if p[0] == "type":
        return f"Box::new(self.{fn_name_of(p[1], p[2])}(&child)?)"
    if p[0] == "shape":
        return "self.shape(&child)?"
    if p[0] == "text":
        return "self.text_body(&child)?"
    if p[0] == "axis":
        return "self.axis_id(&child)?"
    if p[0] == "retained":
        return "self.retain(&child)"
    if p[0] == "simple":
        return "self.choice_text(&child)?" if local_path(p[1].rust, mod) == "String" else f"self.choice_typed::<{local_path(p[1].rust, mod)}>(&child)?"
    raise SystemExit(f"choice_value {p}")


def fn_name_of(mod, name):
    return f"{mod}_{snake(re.sub('^(CT|EG|AG)_', '', name)).rstrip('_')}"


def fn_name(t):
    return fn_name_of(t.mod, t.name)


def any_rows(t):
    return [f"{KEY_OF[t.mod]}:{t.name}/xsd:any"]


def ext_items(builder):
    """Every element an extension can hold that a schema here names."""
    out = []
    for mod in EXT_MODULES:
        m = builder.s.mods[mod]
        for name, node in m.elements.items():
            if name in ROOTS.get(mod, ()):
                continue
            uri, tname = m.q(node.get("type"))
            f = builder.typed(m, name, (uri, tname), mod)
            out.append((mod, name, f, f"{mod.capitalize()}{camel(name)}"))
    # Real decks write `c16:uniqueId` straight under `ext`, where the schema has it only
    # inside `categoryFilterException` (nhs-nda-dpp-1718, z22114038).
    m = builder.s.mods["c16"]
    out.append(("c16", "uniqueId", builder.typed(m, "uniqueId", (m.uris[0], "CT_ChartUniqueID"), "c16"), "C16UniqueId"))
    return out


def emit_ext_model(builder):
    out = [HEADER.format(schema="the extension schemas"),
           "//! What a chart's `ext` can hold. See `tools/chart-codegen/generate.py`.\n\n",
           "use super::{Retained, c, c14, c15, c16, c16r3};\n",
           "use crate::drawing::ShapeProperties;\n\n",
           "/// An item of a `c:CT_Extension` or `cx:CT_Extension`: one a schema here names, or one\n/// nothing here knows, which stays in the tree.\n",
           "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum ExtItem {\n"]
    for mod, name, f, var in ext_items(builder):
        p = f.payload
        if p[0] == "type":
            inner = f"Box<{p[1]}::{type_name(p[2])}>"
        elif p[0] == "shape":
            inner = "Box<ShapeProperties>"
        elif p[0] == "simple":
            inner = p[1].rust.replace("super::", "")
        else:
            raise SystemExit(f"ext {name} {p}")
        out.append(f"    /// `{mod}:{name}`.\n    {var}({inner}),\n")
    out.append("    /// Anything else.\n    Other(Retained),\n}\n")
    return "".join(out)


def emit_ext_read(builder):
    out = [HEADER.format(schema="the extension schemas"),
           "//! What a chart's `ext` can hold. See `tools/chart-codegen/generate.py`.\n\n",
           "use casual_slide_import_prelude::*;\n"]
    out = out[:2] + ["use casual_slide_model::chart::ExtItem;\n\nuse super::{ChartReader, Res, NS_C14, NS_C15, NS_C16, NS_C16AC, NS_C16R2, NS_C16R3};\nuse crate::properties::Elem;\n\n",
                     "impl ChartReader<'_, '_> {\n    /// One item of an `ext`. An item nothing here knows is reported and stays in the tree.\n    pub(crate) fn ext_item(&mut self, el: &Elem) -> Res<ExtItem> {\n"]
    for mod in EXT_MODULES:
        names = [x for x in ext_items(builder) if x[0] == mod]
        out.append(f"        if el.in_namespace(NS_{mod.upper()}) {{\n            match el.local.as_str() {{\n")
        for _, name, f, var in names:
            out.append(f"                // spec-audit: {KEY_OF[mod]}:<{name}>\n")
            p = f.payload
            if p[0] == "type":
                value = f"Box::new(self.{fn_name_of(p[1], p[2])}(el)?)"
            elif p[0] == "shape":
                value = "self.shape(el)?"
            elif p[0] == "simple":
                value = "self.r.text(el)?.unwrap_or_default()"
            out.append(f'                "{name}" => return Ok(ExtItem::{var}({value})),\n')
        out.append("                _ => {}\n            }\n        }\n")
    out.append("        Ok(ExtItem::Other(self.retain(el)))\n    }\n}\n")
    return "".join(out)


def emit_read(builder, mod):
    m = builder.s.mods[mod]
    uses = set()
    for (tm, name), t in builder.types.items():
        if tm != mod:
            continue
        uses.add(t.rust)
        for c in t.choices:
            uses.add(c.enum)
    out = [HEADER.format(schema=pathlib.Path(dict((x[0], x[1]) for x in MODULES)[mod]).name),
           "//! The reader for this namespace's types. See `tools/chart-codegen/generate.py`.\n\n",
           "#[allow(unused_imports)] // Each namespace uses its own few of these.\n",
           "use casual_slide_model::chart::{" + mod + "::*, OpenList, Open};\n"
           "#[allow(unused_imports)]\nuse casual_slide_model::chart::" + "{c, c14, c15, c16, c16ac, c16r2, c16r3, cdr, cdr14, cs, cx};\n"
           "#[allow(unused_imports)]\nuse casual_slide_model::properties::Float64;\n\n"
           "#[allow(unused_imports)]\nuse super::{ChartBound, ChartReader, K, Res, percent_or_number, NS_C, NS_C14, NS_C15, NS_C16, NS_C16AC, NS_C16R2, NS_C16R3, NS_CDR, NS_CDR14, NS_CS, NS_CX, NS_MC};\n"
           "#[allow(unused_imports)]\nuse crate::properties::Elem;\n#[allow(unused_imports)]\nuse crate::properties::ns;\n\n",
           "impl ChartReader<'_, '_> {\n"]
    for (tm, name), t in builder.types.items():
        if tm == mod:
            out.append(emit_read_fn(t, builder, mod).replace("super::", ""))
    out.append("}\n")
    text = "".join(out)
    # Types from another namespace are named through its module.
    for other in builder.s.mods:
        text = text.replace(f"{other}::{other}::", f"{other}::")
    return text


def space_const(name):
    return "dml" if name == "dml" else name.upper()


def write_expr(f, value, name_literal=None):
    """Code producing an N for a value of this field. `value` is a reference expression."""
    p = f.payload
    name = name_literal or f'"{f.local}"'
    sp = space_const(f.space) if f.space else None
    if p[0] == "type":
        return f"super::{p[1]}::{fn_name_of(p[1], p[2])}({value}, {name}, {sp})"
    if p[0] == "axis":
        return f"super::axis_id({value}, {name}, {sp})"
    if p[0] == "shape":
        return f"N::shape({sp}, {name}, {value})"
    if p[0] == "text":
        return f"N::text_body({sp}, {name}, {value})"
    if p[0] == "color":
        return f"N::color({value})"
    if p[0] == "retained":
        return f"N::retained({value})"
    if p[0] == "simple":
        return f"N::text_element({sp}, {name}, {value})"
    raise SystemExit(f"write_expr {p}")


def emit_write_fn(t, builder, mod):
    fn = fn_name(t)
    lines = []
    owners = ",".join(sorted(t.hosts))
    if owners:
        lines.append(f"// spec-audit: owner={owners}\n")
    order = ", ".join(f'"{o}"' for o in t.order)
    if (t.mod, t.name) == ("c", "CT_ChartSpace"):
        # `mc:AlternateContent` is where `style` would be.
        order = order.replace('"style"', '"AlternateContent", "style"')
    if (t.mod, t.name) == ("cdr", "CT_RelSizeAnchor") or (t.mod, t.name) == ("cdr", "CT_AbsSizeAnchor"):
        order = order.replace('"clientData"', '"AlternateContent", "clientData"')
    # A list is a reference to a slice of a value whose elements are read by `.iter()`.
    if (t.mod, t.name) in UNREACHED:
        lines.append("#[allow(dead_code)] // No element in these schemas holds this type, or a hand-written writer writes it.\n")
    empty = not t.fields and (t.mod, t.name) not in HAND_FIELDS
    lines.append(f"pub(crate) fn {fn}({'_v' if empty else 'v'}: &{t.rust}, name: &'static str, space: Space) -> N {{\n")
    lines.append(f"    let {'' if empty else 'mut '}n = N::new(space, name, &[{order}]);\n")
    for f in t.fields:
        if f.kind == "attribute":
            if f.payload[0] == "rel":
                lines.append(f'    n.rel_attr("{f.local}", v.{f.rust}.as_ref());\n')
            elif f.qualified:
                lines.append(f'    n.qualified_attr({space_const(f.qualified)}, "{f.local}", v.{f.rust}.as_ref());\n')
            else:
                lines.append(f'    n.attr("{f.local}", v.{f.rust}.as_ref());\n')
    if (t.mod, t.name) == ("c", "CT_NumVal"):
        lines.append('    n.qualified_attr(C16R2, "formatcode2", v.formatcode2.as_ref());\n')
    for f in t.fields:
        if f.kind == "value":
            lines.append("    n.set_text(v.value.as_ref());\n")
        elif f.kind == "items":
            lines.append("    for x in &v.items {\n        n.push(super::ext_item(x));\n    }\n")
        elif f.kind == "element":
            names_many = f.many
            if f.payload[0] == "color" and not names_many:
                lines.append(f"    if let Some(x) = &v.{f.rust} {{\n        n.push(N::color(x));\n    }}\n")
            elif names_many:
                lines.append(f"    for x in &v.{f.rust} {{\n        n.push({write_expr(f, 'x')});\n    }}\n")
            else:
                lines.append(f"    if let Some(x) = &v.{f.rust} {{\n        n.push({write_expr(f, 'x')});\n    }}\n")
        elif f.kind == "choice":
            lines.append(f"    for x in &v.{f.rust} {{\n        n.push(match x {{\n")
            for b in f.variants:
                lines.append(f"            {f.enum}::{b.variant}(x) => {write_expr(b, 'x')},\n")
            lines.append("        });\n    }\n")
    if (t.mod, t.name) == ("c", "CT_ChartSpace"):
        lines.append("    if let Some(x) = &v.alternate_style {\n        n.push(super::alternate_style(x));\n    }\n")
    if (t.mod, t.name) in (("cdr", "CT_RelSizeAnchor"), ("cdr", "CT_AbsSizeAnchor")):
        lines.append("    for x in &v.alternate_content_part {\n        n.push(super::alternate_content_part(x));\n    }\n")
    lines.append("    n\n}\n\n")
    return "".join(lines)


def emit_write(builder, mod):
    out = [HEADER.format(schema=pathlib.Path(dict((x[0], x[1]) for x in MODULES)[mod]).name),
           "//! What this namespace's values say the tree should hold. See `tools/chart-codegen/generate.py`.\n\n",
           "#[allow(unused_imports)] // Each namespace uses its own few of these.\n",
           f"use casual_slide_model::chart::{mod}::*;\n"
           "#[allow(unused_imports)]\nuse casual_slide_model::chart::{c, c14, c15, c16, c16ac, c16r2, c16r3, cdr, cdr14, cs, cx};\n\n"
           "#[allow(unused_imports)]\nuse super::{C, C14, C15, C16, C16AC, C16R2, C16R3, CDR, CDR14, CS, CX, N, Space, dml};\n\n"]
    for (tm, name), t in builder.types.items():
        if tm == mod:
            out.append(emit_write_fn(t, builder, mod))
    return "".join(out)


def emit_ext_write(builder):
    out = [HEADER.format(schema="the extension schemas"),
           "//! What a chart's `ext` can hold. See `tools/chart-codegen/generate.py`.\n\n",
           "use casual_slide_model::chart::ExtItem;\n\nuse super::{C14, C15, C16, C16AC, C16R2, C16R3, N};\n\n",
           "/// The node for one item of an `ext`.\npub(crate) fn ext_item(item: &ExtItem) -> N {\n    match item {\n"]
    for mod, name, f, var in ext_items(builder):
        out.append(f"        // spec-audit: {KEY_OF[mod]}:<{name}>\n")
        out.append(f"        ExtItem::{var}(x) => {write_expr(f, 'x')},\n")
    out.append("        ExtItem::Other(x) => N::retained(x),\n    }\n}\n")
    return "".join(out)


def outputs():
    builder = Builder(Schemas())
    files = {}
    for mod in URIS_OF:
        files[f"crates/casual-slide-model/src/chart/{mod}.rs"] = emit_model(builder, mod)
        files[f"crates/casual-slide-import/src/chart/{mod}.rs"] = emit_read(builder, mod)
        files[f"crates/casual-slide-export/src/chart/{mod}.rs"] = emit_write(builder, mod)
    files["crates/casual-slide-model/src/chart/ext.rs"] = emit_ext_model(builder)
    files["crates/casual-slide-import/src/chart/ext.rs"] = emit_ext_read(builder)
    files["crates/casual-slide-export/src/chart/ext.rs"] = emit_ext_write(builder)
    return files


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args(argv)
    bad = []
    for path, text in outputs().items():
        target = ROOT / path
        if args.check:
            if not target.exists() or target.read_text(encoding="utf8") != text:
                bad.append(path)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf8")
    for path in bad:
        print(f"{path}: differs from what tools/chart-codegen/generate.py writes", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
