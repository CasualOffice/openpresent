#!/usr/bin/env python3
"""Fidelity check F4: is each XML part of a deck schema-valid?

For each deck: refuse any DOCTYPE, resolve markup compatibility as ECMA-376 Part 3
describes, and validate the result with xmllint against the vendored schemas in
schemas/, reporting per part. xmllint only ever sees text this tool wrote, with no
document type, and runs with --nonet. Files are never extracted under their own names.

    python3 tools/f4/validate.py fixtures/generated fixtures/producers
    python3 tools/f4/validate.py --json deck.pptx
    python3 tools/f4/validate.py --mce-report part.xml   # what the resolver decided
    python3 tools/f4/validate.py --print-tables          # its fixed namespace lists
    python3 tools/f4/validate.py --print-allowances      # where we depart from Ecma's XSD
    python3 tools/f4/validate.py --no-allowances ...     # Ecma's Transitional XSD unchanged
    python3 tools/f4/validate.py --no-known ...          # recorded failures fail

Status of a part: valid; invalid; refused (DOCTYPE or over a bound); malformed;
must-understand (a namespace we do not know, per Part 3); no-schema (nothing vendored
for its root, so it was not checked, which is reported and is not a pass);
schema-error (the schema itself would not compile); recorded (invalid, but every message
is one recorded-failures.tsv explains).

Transitional parts validate against the vendored XSD with allowances.tsv applied, in a
private copy: PowerPoint writes things Ecma's XSD refuses (F4-02, F4-03). Each allowance
is one exact edit that must apply as many times as it says, or the tool stops. Strict is
never relaxed. schemas/ itself is not changed.

Exit: 0 if nothing failed, 1 if a deck or part failed, 2 if the tool could not run.

The rules mirror crates/casual-slide-ooxml/src/mce.rs, and the tables mirror
namespaces.rs. tests/corpus.rs holds the two together on the same inputs. Python here,
since the engine has no business shelling out to a validator.
"""
import atexit
import fnmatch
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import xml.parsers.expat
import zipfile
import zlib
from xml.sax.saxutils import escape, quoteattr

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCHEMAS = ROOT / "schemas"
HERE = pathlib.Path(__file__).resolve().parent
CATALOG = HERE / "catalog.xml"
ALLOWANCES = HERE / "allowances.tsv"
RECORDED = HERE / "recorded-failures.tsv"
DECK_SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}

# Bounds, as OoxmlLimits has them or tighter. A validator is not a reason to be less careful.
MAX_ENTRIES = 100_000
MAX_PART_BYTES = 256 << 20
MAX_RATIO = 1_000
MAX_ELEMENTS = 5_000_000
MAX_DEPTH = 256
MAX_REPORTED_ERRORS = 20

MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
XML_NS = "http://www.w3.org/XML/1998/namespace"
OPC_RELS = "http://schemas.openxmlformats.org/package/2006/relationships"
OPC_TYPES = "http://schemas.openxmlformats.org/package/2006/content-types"
OPC_CORE = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties"

# (Strict, Transitional, schema file name in schemas/ooxml-strict and schemas/ooxml or None).
_S = "http://purl.oclc.org/ooxml/"
_T = "http://schemas.openxmlformats.org/"
NAMESPACES = [
    (_S + "presentationml/main", _T + "presentationml/2006/main", "pml.xsd"),
    (_S + "drawingml/main", _T + "drawingml/2006/main", "dml-main.xsd"),
    (_S + "drawingml/chart", _T + "drawingml/2006/chart", "dml-chart.xsd"),
    (_S + "drawingml/chartDrawing", _T + "drawingml/2006/chartDrawing", "dml-chartDrawing.xsd"),
    (_S + "drawingml/diagram", _T + "drawingml/2006/diagram", "dml-diagram.xsd"),
    (_S + "drawingml/lockedCanvas", _T + "drawingml/2006/lockedCanvas", "dml-lockedCanvas.xsd"),
    (_S + "drawingml/picture", _T + "drawingml/2006/picture", "dml-picture.xsd"),
    (_S + "drawingml/spreadsheetDrawing", _T + "drawingml/2006/spreadsheetDrawing", None),
    (_S + "drawingml/wordprocessingDrawing", _T + "drawingml/2006/wordprocessingDrawing", None),
    (_S + "officeDocument/relationships", _T + "officeDocument/2006/relationships", "shared-relationshipReference.xsd"),
    (_S + "officeDocument/sharedTypes", _T + "officeDocument/2006/sharedTypes", "shared-commonSimpleTypes.xsd"),
    (_S + "officeDocument/math", _T + "officeDocument/2006/math", None),
    (_S + "officeDocument/customXml", _T + "officeDocument/2006/customXml", "shared-customXmlDataProperties.xsd"),
    (_S + "officeDocument/customProperties", _T + "officeDocument/2006/custom-properties", "shared-documentPropertiesCustom.xsd"),
    (_S + "officeDocument/extendedProperties", _T + "officeDocument/2006/extended-properties", "shared-documentPropertiesExtended.xsd"),
    (_S + "officeDocument/docPropsVTypes", _T + "officeDocument/2006/docPropsVTypes", "shared-documentPropertiesVariantTypes.xsd"),
    (_S + "officeDocument/bibliography", _T + "officeDocument/2006/bibliography", None),
    (_S + "officeDocument/characteristics", _T + "officeDocument/2006/characteristics", None),
    (_S + "schemaLibrary/main", _T + "schemaLibrary/2006/main", "shared-customXmlSchemaProperties.xsd"),
    (_S + "spreadsheetml/main", _T + "spreadsheetml/2006/main", None),
    (_S + "wordprocessingml/main", _T + "wordprocessingml/2006/main", None),
]
OTHER_UNDERSTOOD = [
    OPC_RELS,
    OPC_TYPES,
    OPC_CORE,
    "http://purl.org/dc/elements/1.1/",
    "http://purl.org/dc/terms/",
    "http://purl.org/dc/dcmitype/",
    "http://www.w3.org/2001/XMLSchema-instance",
    "urn:schemas-microsoft-com:vml",
    "urn:schemas-microsoft-com:office:office",
    "urn:schemas-microsoft-com:office:powerpoint",
]
UNDERSTOOD = {MC, XML_NS, *OTHER_UNDERSTOOD, *(n for row in NAMESPACES for n in row[:2])}

# Root namespace to schema. Roots with no entry are reported as no-schema.
SCHEMA_FOR_ROOT = {OPC_RELS: "opc/opc-relationships.xsd", OPC_TYPES: "opc/opc-contentTypes.xsd", OPC_CORE: "../tools/f4/coreProperties.xsd"}
# VML (F4-04). vml-main.xsd imports wml.xsd and three more, vendored unmodified for this.
VML = "urn:schemas-microsoft-com:vml"
VML_OFFICE = "urn:schemas-microsoft-com:office:office"
VML_POWERPOINT = "urn:schemas-microsoft-com:office:powerpoint"
SCHEMA_FOR_ROOT[VML] = "ooxml/vml-main.xsd"
SCHEMA_FOR_ROOT[VML_OFFICE] = "ooxml/vml-officeDrawing.xsd"
SCHEMA_FOR_ROOT[VML_POWERPOINT] = "ooxml/vml-presentationDrawing.xsd"
for _strict, _transitional, _file in NAMESPACES:
    if _file:
        SCHEMA_FOR_ROOT[_strict] = f"ooxml-strict/{_file}"
        SCHEMA_FOR_ROOT[_transitional] = f"ooxml/{_file}"
# Charts from Office 2016 and chart styles (F4-05): Microsoft's own schemas, from MS-ODRAWXML.
SCHEMA_FOR_ROOT["http://schemas.microsoft.com/office/drawing/2014/chartex"] = "ms-extensions/ms-odrawxml-22.xsd"
SCHEMA_FOR_ROOT["http://schemas.microsoft.com/office/drawing/2012/chartStyle"] = "ms-extensions/ms-odrawxml-15.xsd"

# OPC forbids markup compatibility in these parts, so it is not resolved in them, and
# the schema rejects it if it is there.
NO_COMPATIBILITY = {OPC_RELS, OPC_TYPES, OPC_CORE}

# Verdicts from least to most serious, for a part checked as several fragments.
SEVERITY = ["", "valid", "recorded", "invalid", "schema-error"]

LIST_SEPARATOR = re.compile(r"[ \t\n\r\f]+")


def stop(message):
    """The tool could not run, which is exit 2: a deck did not fail."""
    print(f"f4: {message}", file=sys.stderr)
    raise SystemExit(2)


def tsv_rows(path, columns):
    """A hand-kept table's rows, as dicts. A row of the wrong shape stops the tool."""
    rows = []
    for line in path.read_text(encoding="utf8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        cols = line.split("\t")
        if len(cols) != len(columns):
            stop(f"{path.name}: a row needs {len(columns)} tab-separated columns: {line[:60]}")
        rows.append(dict(zip(columns, cols)))
    return rows


def allowances():
    return tsv_rows(ALLOWANCES, ("id", "file", "count", "old", "new", "evidence", "reason"))


def apply_allowances(texts, rows):
    """The XSD files' text with each edit made. An edit that does not apply exactly as
    often as it says means the schemas moved under it, so nothing is validated."""
    for row in rows:
        old, new = (row[k].replace("\\n", "\r\n") for k in ("old", "new"))
        found = texts.get(row["file"], "").count(old)
        if found != int(row["count"]):
            stop(f"allowance {row['id']}: expected {row['count']} matches in {row['file']}, found {found}")
        texts[row["file"]] = texts[row["file"]].replace(old, new)
    return texts


def extension_locations():
    """Namespace to schema file, as a path from schemas/ms-extensions, for every namespace
    vendored: Ecma's Transitional files, then Microsoft's by index.json."""
    where = {t: f"../ooxml/{f}" for _, t, f in NAMESPACES if f}
    index = json.loads((SCHEMAS / "ms-extensions/index.json").read_text(encoding="utf8"))
    where.update((uri, name) for name, uri, *_ in index)
    return where


IMPORT = re.compile(r"<xsd:import\b[^>]*/>")
BETWEEN_TAGS = re.compile(r">[^<]*<")
QNAME_ATTRIBUTE = re.compile(r'\b(type|ref|base|itemType)="\s*([^"\s]+)\s*"')


def locate_imports(text, where=None):
    """Microsoft's schemas import by Microsoft's own file names (`oartbasetypes.xsd`,
    `orel.xsd`), which are not published as files. Each import's location becomes the
    vendored file for its namespace; nothing else changes, so nothing is relaxed (F4-05).
    An import of a namespace nothing vendored keeps its location, and xmllint says so.
    Several of Microsoft's files for one namespace are one Ecma file, so it is imported once:
    xmllint will not compile a second import of a location it has."""
    where = extension_locations() if where is None else where
    seen = set()

    def one(match):
        tag = match.group(0)
        namespace = re.search(r'namespace="([^"]*)"', tag)
        if not namespace or namespace.group(1) not in where:
            return tag
        if namespace.group(1) in seen:
            return ""
        seen.add(namespace.group(1))
        located = f'schemaLocation="{where[namespace.group(1)]}"'
        if "schemaLocation=" in tag:
            return re.sub(r'schemaLocation="[^"]*"', located, tag)
        return tag[:-2].rstrip() + f" {located}/>"

    return IMPORT.sub(one, text)


XML_IMPORT = '<xsd:import namespace="http://www.w3.org/XML/1998/namespace"/>'


class Schemas:
    """Where each schema is read from. Transitional is a private copy with the allowances."""

    use_allowances = True
    _overlay = None

    @classmethod
    def path(cls, schema):
        if not schema.startswith(("ooxml/", "ms-extensions/")):
            return SCHEMAS / schema
        if cls._overlay is None:
            cls._overlay = cls._build(cls.use_allowances)
        return cls._overlay / schema

    @staticmethod
    def _build(use_allowances):
        work = pathlib.Path(tempfile.mkdtemp(prefix="f4-schemas-"))
        atexit.register(shutil.rmtree, work, ignore_errors=True)
        texts = {p.name: p.read_bytes().decode("utf8") for p in (SCHEMAS / "ooxml").glob("*.xsd")}
        # Not an allowance: wml.xsd and shared-math.xsd import the xml: namespace with no
        # location, and xmllint will not compile them without one. The W3C's own xml.xsd
        # is given, through catalog.xml, and nothing is relaxed (F4-04).
        apply_allowances(texts, [{"id": "xml-namespace-location", "file": name, "count": "1", "old": XML_IMPORT,
                                  "new": XML_IMPORT[:-2] + ' schemaLocation="http://www.w3.org/2001/xml.xsd"/>'}
                                 for name in ("wml.xsd", "shared-math.xsd")])
        for source in (SCHEMAS / "ms-extensions").glob("*.xsd"):
            text = source.read_bytes().decode("utf8")
            # Extracted from web pages, these indent with no-break spaces, which XML does
            # not count as white space, so no element compiles. Between tags only.
            text = BETWEEN_TAGS.sub(lambda m: m.group(0).replace("\u00a0", " "), text)
            # A QName collapses white space (XSD Part 2, 3.2.18), which xmllint does not do:
            # MS-ODRAWXML writes `type="CT_GeoHierarchyEntity "`.
            text = QNAME_ATTRIBUTE.sub(r'\1="\2"', text)
            texts[f"ms-extensions/{source.name}"] = locate_imports(text)
        if use_allowances:
            apply_allowances(texts, allowances())
        (work / "ms-extensions").mkdir()
        (work / "ooxml").mkdir()
        for name, text in texts.items():
            target = work / name if name.startswith("ms-extensions/") else work / "ooxml" / name
            target.write_bytes(text.encode("utf8"))
        return work


def recorded_failures():
    return tsv_rows(RECORDED, ("deck", "part", "message", "kind", "reason"))


def explained_by(rows, deck, part, message):
    """Does a recorded row cover this one message of this part?"""
    return any(fnmatch.fnmatch(deck, row["deck"]) and fnmatch.fnmatch(part, row["part"])
               and re.search(row["message"], message) for row in rows)


class Refused(Exception):
    """A part this tool will not process, with a reason that carries no content."""


class Node:
    __slots__ = ("name", "attrs", "children", "parent")

    def __init__(self, name, attrs, parent=None):
        self.name = name
        self.attrs = attrs
        self.children = []
        self.parent = parent


def refuse_doctype(data):
    """Before any parser sees the bytes. Conservative: a comment that mentions one counts."""
    if data[:2] in (b"\xff\xfe", b"\xfe\xff"):
        text = data.decode("utf-16", "replace")
    else:
        text = data.decode("latin-1")
    if "<!doctype" in text.lower() or "<!entity" in text.lower():
        raise Refused("DOCTYPE")


def parse(data):
    """Bytes to a tree, under the bounds, and refusing a document type again in the parser."""
    refuse_doctype(data)
    parser = xml.parsers.expat.ParserCreate()
    parser.buffer_text = True
    parser.ordered_attributes = True
    parser.SetParamEntityParsing(xml.parsers.expat.XML_PARAM_ENTITY_PARSING_NEVER)
    state = {"elements": 0}
    stack = []
    roots = []

    def start(name, flat):
        state["elements"] += 1
        if state["elements"] > MAX_ELEMENTS:
            raise Refused("too many elements")
        if len(stack) >= MAX_DEPTH:
            raise Refused("too deep")
        node = Node(name, list(zip(flat[::2], flat[1::2])), stack[-1] if stack else None)
        (stack[-1].children if stack else roots).append(node)
        stack.append(node)

    def end(_name):
        stack.pop()

    def text(data):
        if stack:
            stack[-1].children.append(data)

    def doctype(*_args):
        raise Refused("DOCTYPE")

    parser.StartElementHandler = start
    parser.EndElementHandler = end
    parser.CharacterDataHandler = text
    parser.StartDoctypeDeclHandler = doctype
    parser.EntityDeclHandler = doctype
    parser.Parse(data, True)
    return roots[0]


def split_list(value):
    return [v for v in LIST_SEPARATOR.split(value) if v]


class Report:
    """What Part 3 says about one part. Mirrors `Compatibility` in mce.rs."""

    def __init__(self):
        self.alternates = []
        self.ignored = 0
        self.unwrapped = 0
        self.ignored_attributes = 0
        self.extensions = 0
        self.must_understand = set()
        self.unknown = set()
        self.nonconformances = set()

    def line(self):
        return " ".join([
            "alternates=" + ",".join(self.alternates),
            "must_understand=" + "|".join(sorted(self.must_understand)),
            "unknown=" + "|".join(sorted(self.unknown)),
            "nonconformances=" + ",".join(sorted(self.nonconformances)),
            f"ignored={self.ignored}",
            f"unwrapped={self.unwrapped}",
            f"ignored_attributes={self.ignored_attributes}",
            f"extensions={self.extensions}",
        ])


class Resolver:
    """ECMA-376 Part 3, clause 9, building the output document. The input is not changed."""

    def __init__(self):
        self.prefixes = {}
        self.ignorable = {}
        self.process = {}
        self.report = Report()

    @staticmethod
    def knows(uri):
        return uri in UNDERSTOOD

    def lookup(self, prefix):
        if prefix == "xml":
            return XML_NS
        stack = self.prefixes.get(prefix)
        return (stack[-1] or None) if stack else None

    def list_entry(self, prefix):
        uri = self.lookup(prefix)
        if uri is None:
            self.report.nonconformances.add("undeclared-prefix")
            return None
        if uri == MC:
            self.report.nonconformances.add("lists-compatibility-namespace")
            return None
        return uri

    def enter(self, node):
        undo = ([], [], [])
        for name, value in node.attrs:
            if name == "xmlns" or name.startswith("xmlns:"):
                prefix = name[6:]
                self.prefixes.setdefault(prefix, []).append(value)
                undo[0].append(prefix)
        attributes = []
        for name, value in node.attrs:
            if name == "xmlns" or name.startswith("xmlns:"):
                continue
            prefix, colon, local = name.partition(":")
            if colon:
                attributes.append((self.lookup(prefix) or "", local, value, name))
            else:
                attributes.append((None, name, value, name))
        for namespace, local, value, _ in attributes:
            if namespace == MC and local == "Ignorable":
                for prefix in split_list(value):
                    uri = self.list_entry(prefix)
                    if uri is not None:
                        self.ignorable[uri] = self.ignorable.get(uri, 0) + 1
                        undo[1].append(uri)
        for namespace, local, value, _ in attributes:
            if namespace == MC and local == "ProcessContent":
                for token in split_list(value):
                    prefix, colon, name = token.partition(":")
                    if not colon or not prefix or not name or ":" in name:
                        self.report.nonconformances.add("malformed-process-content")
                        continue
                    uri = self.list_entry(prefix)
                    if uri is None:
                        continue
                    if uri not in self.ignorable:
                        self.report.nonconformances.add("process-content-not-ignorable")
                        continue
                    pair = (uri, name)
                    self.process[pair] = self.process.get(pair, 0) + 1
                    undo[2].append(pair)
        return attributes, undo

    def leave(self, undo):
        for prefix in undo[0]:
            self.prefixes[prefix].pop()
        for table, keys in ((self.ignorable, undo[1]), (self.process, undo[2])):
            for key in keys:
                table[key] -= 1
                if not table[key]:
                    del table[key]

    def name_of(self, node):
        prefix, colon, local = node.name.partition(":")
        if colon:
            uri = self.lookup(prefix)
            if uri is None:
                self.report.nonconformances.add("undeclared-element-prefix")
            return uri, local
        return self.lookup(""), node.name

    def check_must_understand(self, attributes):
        for namespace, local, value, _ in attributes:
            if namespace == MC and local == "MustUnderstand":
                for prefix in split_list(value):
                    uri = self.list_entry(prefix)
                    if uri is not None and not self.knows(uri):
                        self.report.must_understand.add(uri)

    def visit_children(self, node, out, inherited):
        for child in node.children:
            if isinstance(child, str):
                out.append(child)
            else:
                self.visit(child, out, inherited)

    def visit(self, node, out, inherited):
        attributes, undo = self.enter(node)
        self.visit_entered(node, attributes, out, inherited)
        self.leave(undo)

    def hoisted(self, node, inherited):
        """Declarations on an element that is going away, for the content that keeps it."""
        merged = dict(inherited)
        merged.update((n, v) for n, v in node.attrs if n == "xmlns" or n.startswith("xmlns:"))
        return merged

    def visit_entered(self, node, attributes, out, inherited):
        namespace, local = self.name_of(node)
        report = self.report

        if namespace is not None and not self.knows(namespace) and namespace in self.ignorable:
            if (namespace, local) not in self.process and (namespace, "*") not in self.process:
                report.ignored += 1
                return
            report.unwrapped += 1
            self.check_must_understand(attributes)
            if any(a[0] == XML_NS and a[1] in ("base", "lang", "space") for a in attributes):
                report.nonconformances.add("unwrapped-with-xml-attribute")
            self.visit_children(node, out, self.hoisted(node, inherited))
            return

        if namespace == MC:
            if local == "AlternateContent":
                self.alternate(node, attributes, out, inherited)
                return
            if local in ("Choice", "Fallback"):
                report.nonconformances.add("misplaced-branch")
            else:
                report.nonconformances.add("unknown-compatibility-element")
        elif namespace is not None and not self.knows(namespace):
            report.unknown.add(namespace)

        if (local == "ext" and namespace is not None and self.knows(namespace)
                and node.parent is not None and node.parent.name.rpartition(":")[2] == "extLst"):
            report.extensions += 1
            copy = Node(node.name, with_inherited(node.attrs, inherited))
            copy.children = node.children
            out.append(copy)
            return

        self.check_must_understand(attributes)
        kept = [(n, v) for n, v in node.attrs if n == "xmlns" or n.startswith("xmlns:")]
        for attr_ns, attr_local, value, name in attributes:
            if attr_ns is None or attr_ns == "":
                kept.append((name, value))
            elif attr_ns == MC:
                if attr_local not in ("Ignorable", "ProcessContent", "MustUnderstand", "PreserveElements", "PreserveAttributes"):
                    report.nonconformances.add("unknown-compatibility-attribute")
            elif self.knows(attr_ns):
                kept.append((name, value))
            elif attr_ns in self.ignorable:
                report.ignored_attributes += 1
            else:
                report.unknown.add(attr_ns)
                kept.append((name, value))
        copy = Node(node.name, with_inherited(kept, inherited))
        self.visit_children(node, copy.children, {})
        out.append(copy)

    def check_branch_attributes(self, attributes, allowed):
        for namespace, local, _, _ in attributes:
            if namespace is None:
                ok = local == allowed
            elif namespace == MC:
                ok = True
            elif namespace in (XML_NS, ""):
                ok = False
            else:
                ok = namespace in self.ignorable
            if not ok:
                self.report.nonconformances.add("bad-compatibility-element-attribute")

    def requires_satisfied(self, attributes):
        requires = next((a[2] for a in attributes if a[0] is None and a[1] == "Requires"), "")
        prefixes = split_list(requires)
        satisfied = True
        for prefix in prefixes:
            uri = self.list_entry(prefix)
            if uri is None or not self.knows(uri):
                satisfied = False
        if not prefixes:
            self.report.nonconformances.add("missing-requires")
        return bool(prefixes) and satisfied

    def alternate(self, node, attributes, out, inherited):
        report = self.report
        self.check_branch_attributes(attributes, "")
        self.check_must_understand(attributes)
        choices = 0
        fallback_seen = False
        selected = None
        decision = "none"
        for child in node.children:
            if isinstance(child, str):
                continue
            child_attributes, undo = self.enter(child)
            namespace, local = self.name_of(child)
            if namespace == MC and local == "Choice":
                index = choices
                choices += 1
                self.check_branch_attributes(child_attributes, "Requires")
                if fallback_seen:
                    report.nonconformances.add("branch-out-of-order")
                satisfied = self.requires_satisfied(child_attributes)
                if selected is None and not fallback_seen and satisfied:
                    selected = child
                    decision = f"choice:{index}"
            elif namespace == MC and local == "Fallback":
                self.check_branch_attributes(child_attributes, "")
                if fallback_seen:
                    report.nonconformances.add("branch-out-of-order")
                if selected is None and not fallback_seen:
                    selected = child
                    decision = "fallback"
                fallback_seen = True
            elif namespace is not None and not self.knows(namespace) and namespace in self.ignorable:
                report.ignored += 1
            else:
                report.nonconformances.add("unexpected-child")
            self.leave(undo)
        if choices == 0:
            report.nonconformances.add("no-choice")
        report.alternates.append(decision)
        if selected is not None:
            branch_attributes, undo = self.enter(selected)
            self.check_must_understand(branch_attributes)
            hoisted = self.hoisted(selected, self.hoisted(node, inherited))
            self.visit_children(selected, out, hoisted)
            self.leave(undo)


def with_inherited(attrs, inherited):
    own = {name for name, _ in attrs}
    return [(n, v) for n, v in inherited.items() if n not in own] + list(attrs)


def resolve(root):
    """The output document's top-level nodes, and the report."""
    resolver = Resolver()
    out = []
    resolver.visit(root, out, {})
    return [n for n in out if isinstance(n, Node)], resolver.report


def serialize(node, out):
    out.append(f"<{node.name}")
    for name, value in node.attrs:
        out.append(f" {name}={quoteattr(value)}")
    if not node.children:
        out.append("/>")
        return
    out.append(">")
    for child in node.children:
        if isinstance(child, str):
            out.append(escape(child))
        else:
            serialize(child, out)
    out.append(f"</{node.name}>")


def root_namespace(node):
    prefix, colon, _ = node.name.partition(":")
    wanted = f"xmlns:{prefix}" if colon else "xmlns"
    return next((v for n, v in node.attrs if n == wanted), None)


def redact(message):
    """xmllint quotes values from the document. Diagnostics carry names, not content."""
    message = re.sub(r"The value '[^']*'", "The value '...'", message)
    message = re.sub(r": '[^']*' is not a valid value", ": '...' is not a valid value", message)
    return message.strip()


def run_xmllint(schema, jobs, workdir):
    """Validate many files against one schema, in one process: compiling is the slow part.
    Returns {path: (status, [messages])}."""
    environment = {"PATH": os.environ.get("PATH", ""), "XML_CATALOG_FILES": str(CATALOG)}
    # Bare names, run in the directory that holds them. xmllint reports a file under the
    # name it parsed, and on Windows it writes a full path with `/` for `\`, so a full
    # path given here matched its verdict but none of its messages.
    paths = [pathlib.Path(p).name for p in jobs]
    assert all(pathlib.Path(workdir) / name == pathlib.Path(p) for name, p in zip(paths, jobs))
    done = subprocess.run(
        ["xmllint", "--noout", "--nonet", "--schema", str(Schemas.path(schema)), *paths],
        capture_output=True, text=True, errors="replace", env=environment, cwd=workdir, timeout=600, check=False)
    output = done.stdout + done.stderr
    results = {}
    if "failed to compile" in output:
        message = redact(output)[-400:]
        return {str(pathlib.Path(workdir) / p): ("schema-error", [message]) for p in paths}
    messages = {p: [] for p in paths}
    for line in output.splitlines():
        for p in paths:
            if line == f"{p} validates":
                results[p] = ("valid", [])
            elif line == f"{p} fails to validate":
                results[p] = ("invalid", messages[p])
            elif line.startswith(f"{p}:"):
                messages[p].append(redact(line[len(p) + 1:]))
            else:
                continue
            break
    for p in paths:
        results.setdefault(p, ("schema-error", [f"xmllint gave no verdict, exit {done.returncode}"]))
    return {str(pathlib.Path(workdir) / name): result for name, result in results.items()}


def is_xml_part(name):
    lower = name.lower()
    return lower.endswith(".xml") or lower.endswith(".rels") or lower.endswith(".vml")


def vml_fragments(top):
    """A VML drawing part is a bare `<xml>` wrapper that no schema defines, holding VML
    elements. Each element is checked on its own, carrying the wrapper's declarations;
    text between them is whitespace or nothing a schema could judge. None if `top` is not
    such a wrapper."""
    if top.name != "xml" or root_namespace(top) is not None:
        return None
    declared = [(n, v) for n, v in top.attrs if n.startswith("xmlns:")]
    return [_rooted(child, declared)
            for child in top.children if isinstance(child, Node)]


def _rooted(child, declared):
    copy = Node(child.name, with_inherited(child.attrs, dict(declared)))
    copy.children = child.children
    return copy


def read_parts(path):
    """Every XML part's bytes, by name, under the admission bounds. Names are not trusted
    as paths: nothing is extracted under them."""
    data = pathlib.Path(path).read_bytes()
    if data[:8] == bytes.fromhex("d0cf11e0a1b11ae1"):
        raise Refused("a Compound File")
    parts = {}
    other = 0
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
            if len(infos) > MAX_ENTRIES:
                raise Refused("too many entries")
            names = [i.filename for i in infos]
            if len(set(names)) != len(names):
                raise Refused("duplicate entries")
            for info in infos:
                if info.is_dir():
                    continue
                if not is_xml_part(info.filename):
                    other += 1
                    continue
                if info.file_size > MAX_PART_BYTES:
                    raise Refused("a part over the size bound")
                if info.compress_size and info.file_size // info.compress_size > MAX_RATIO:
                    raise Refused("a part over the expansion bound")
                parts[info.filename] = archive.read(info)
    except (zipfile.BadZipFile, RuntimeError, NotImplementedError, EOFError, OSError, zlib.error) as err:
        raise Refused(f"not a readable package: {type(err).__name__}") from err
    return parts, other


def validate_deck(path, recorded=()):
    deck = pathlib.Path(path).name
    try:
        parts, other = read_parts(path)
    except Refused as err:
        return {"deck": deck, "status": "fail", "refused": str(err), "parts": [], "not_xml": 0}

    results = []
    jobs = {}
    with tempfile.TemporaryDirectory(prefix="f4-") as workdir:
        by_file = {}
        for number, name in enumerate(sorted(parts)):
            entry = {"part": name, "status": "", "schema": None, "branches": [], "nonconformances": [], "unknown": [], "errors": [], "error_count": 0}
            results.append(entry)
            try:
                root = parse(parts[name])
            except Refused as err:
                entry.update(status="refused", errors=[str(err)], error_count=1)
                continue
            except xml.parsers.expat.ExpatError as err:
                entry.update(status="malformed", errors=[xml.parsers.expat.ErrorString(err.code)], error_count=1)
                continue
            if root_namespace(root) in NO_COMPATIBILITY:
                tops = [root]
            else:
                tops, report = resolve(root)
                entry.update(branches=report.alternates, nonconformances=sorted(report.nonconformances), unknown=sorted(report.unknown))
                if report.must_understand:
                    entry.update(status="must-understand", errors=sorted(report.must_understand), error_count=len(report.must_understand))
                    continue
            if len(tops) != 1:
                entry.update(status="invalid", errors=[f"{len(tops)} root elements after markup compatibility"], error_count=1)
                continue
            fragments = vml_fragments(tops[0])
            if fragments is None:
                fragments = tops
            elif not fragments:
                entry.update(status="no-schema")
                continue
            schemas = [SCHEMA_FOR_ROOT.get(root_namespace(f)) for f in fragments]
            if None in schemas:
                entry.update(status="no-schema")
                continue
            entry["schema"] = ",".join(sorted(set(schemas)))
            for index, (fragment, schema) in enumerate(zip(fragments, schemas)):
                text = []
                serialize(fragment, text)
                target = pathlib.Path(workdir) / f"p{number:06}-{index:04}.xml"
                target.write_text('<?xml version="1.0" encoding="UTF-8"?>\n' + "".join(text), encoding="utf8")
                jobs.setdefault(schema, []).append(target)
                by_file[str(target)] = entry
        for schema, files in sorted(jobs.items()):
            for target, (status, messages) in run_xmllint(schema, files, workdir).items():
                entry = by_file[target]
                # Judged on every message, not only the ones kept for the report.
                if status == "invalid" and messages and all(
                        explained_by(recorded, deck, entry["part"], m) for m in messages):
                    status = "recorded"
                # A VML part is several fragments: the worst verdict stands, messages add up.
                if entry["status"]:
                    status = max(entry["status"], status, key=SEVERITY.index)
                    messages = entry.setdefault("_all", []) + messages
                entry["_all"] = messages
                entry.update(status=status, errors=messages[:MAX_REPORTED_ERRORS], error_count=len(messages))
        for entry in results:
            entry.pop("_all", None)
    failed = any(e["status"] not in ("valid", "no-schema", "recorded") for e in results)
    return {"deck": deck, "status": "fail" if failed else "pass", "refused": None, "parts": results, "not_xml": other}


def expand(arguments):
    for argument in arguments:
        path = pathlib.Path(argument)
        # Hostile fixtures exist for the engine's own parsers. No external tool runs on them.
        if "hostile" in path.resolve().parts and "fixtures" in path.resolve().parts:
            raise SystemExit("f4: refusing fixtures/hostile: no external tool runs on it")
        if path.is_dir():
            yield from sorted(p for p in path.iterdir() if p.suffix.lower() in DECK_SUFFIXES)
        else:
            yield path


def print_text(result):
    print(f"{result['deck']}: {result['status'].upper()}" + (f" (refused: {result['refused']})" if result["refused"] else ""))
    for entry in result["parts"]:
        schema = f"  [{entry['schema']}]" if entry["schema"] else ""
        branches = f"  branches={','.join(entry['branches'])}" if entry["branches"] else ""
        print(f"  {entry['status']:<16}{entry['part']}{schema}{branches}")
        for error in entry["errors"]:
            print(f"      {error}")
        if entry["error_count"] > len(entry["errors"]):
            print(f"      ... and {entry['error_count'] - len(entry['errors'])} more")
        if entry["nonconformances"]:
            print(f"      markup compatibility: {', '.join(entry['nonconformances'])}")
    print(f"  ({result['not_xml']} parts not XML, not checked)")


def main(argv):
    if argv[:1] == ["--print-tables"]:
        for strict, transitional, _ in NAMESPACES:
            print(f"map\t{strict}\t{transitional}")
        for uri in sorted(UNDERSTOOD):
            print(f"understood\t{uri}")
        return 0
    if argv[:1] == ["--mce-report"] and len(argv) == 2:
        try:
            _, report = resolve(parse(pathlib.Path(argv[1]).read_bytes()))
        except Refused as err:
            raise SystemExit(f"f4: refused: {err}") from err
        print(report.line())
        return 0
    if argv[:1] == ["--print-allowances"]:
        for row in allowances():
            print(f"{row['id']}\t{row['file']}\t{row['reason']}")
        return 0
    options = {a for a in argv if a.startswith("-")}
    arguments = [a for a in argv if not a.startswith("-")]
    as_json = "--json" in options
    Schemas.use_allowances = "--no-allowances" not in options
    recorded = [] if "--no-known" in options else recorded_failures()
    if not arguments or options - {"--json", "--no-allowances", "--no-known"}:
        print(__doc__, file=sys.stderr)
        return 2
    if shutil.which("xmllint") is None:
        print("f4: xmllint is not installed", file=sys.stderr)
        return 2
    results = [validate_deck(path, recorded) for path in expand(arguments)]
    if as_json:
        print(json.dumps(results, indent=1, sort_keys=True))
    else:
        for result in results:
            print_text(result)
        failed = sum(r["status"] == "fail" for r in results)
        print(f"{len(results)} decks, {failed} failed")
    if any(e["status"] == "schema-error" for r in results for e in r["parts"]):
        return 2
    return 1 if any(r["status"] == "fail" for r in results) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
