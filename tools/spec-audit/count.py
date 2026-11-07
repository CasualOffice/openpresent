#!/usr/bin/env python3
"""SPEC-03: count schema constructs independently and compare with the checklist.

The generator walks an ElementTree and yields rows. This reads the same XSD files
as an event stream (pyexpat), shares no code with it, and counts every declaration
wherever it occurs, so a construct the generator's walk does not reach shows up as
a difference instead of silently having no row.

Also fails if a schema on disk is neither a checklist source nor named as excluded,
which would otherwise make both counts agree while a file is missing from each.

    python3 tools/spec-audit/count.py            # table; non-zero on any mismatch
    python3 tools/spec-audit/count.py --root DIR --checklist FILE
"""
import argparse
import csv
import json
import pathlib
import re
import sys
import xml.parsers.expat
from collections import Counter

XS = "http://www.w3.org/2001/XMLSchema"
HAND_WRITTEN_PREFIX = "mc:"  # rows no schema gives (SPEC-08); this tool counts schemas
KINDS = ("root", "rootattr", "type", "simple", "value", "element", "attribute",
         "group", "attrgroup", "any", "base")
csv.field_size_limit(1 << 24)


def count_schema(path):
    """Declarations in one XSD, as a Counter keyed like the checklist's `kind` column.

    `inline-values` are enumerations written inside an attribute's own simpleType:
    the checklist holds those in the attribute row's restriction column, not as rows.
    `nested-*` are declarations inside an element that is itself inside a type: legal
    XSD, and exactly what a walk that stops at the element would miss.
    """
    counts = Counter()
    stack = []

    def start(tag, attrs):
        uri, _, name = tag.rpartition(" ")
        if uri != XS:
            stack.append(None)
            return
        depth = len(stack)  # 0 is xsd:schema itself
        path_tags = [t for t in stack if t]
        nested = "element" in path_tags[2:]  # past the top-level declaration itself
        if name == "element" and (attrs.get("name") or attrs.get("ref")):
            if depth == 1:
                counts["root"] += 1
            else:
                counts["nested-element" if nested else "element"] += 1
        elif name == "attribute" and (attrs.get("name") or attrs.get("ref")):
            if depth == 1:
                counts["rootattr"] += 1
            else:
                counts["nested-attribute" if nested else "attribute"] += 1
        elif name in ("complexType", "group", "attributeGroup") and depth == 1 and attrs.get("name"):
            counts["type"] += 1
        elif name == "simpleType" and depth == 1 and attrs.get("name"):
            counts["simple"] += 1
        elif name == "group" and attrs.get("ref"):
            counts["nested-group" if nested else "group"] += 1
        elif name == "attributeGroup" and attrs.get("ref"):
            counts["nested-attrgroup" if nested else "attrgroup"] += 1
        elif name in ("any", "anyAttribute"):
            counts["nested-any" if nested else "any"] += 1
        elif name in ("extension", "restriction") and "simpleType" not in path_tags and attrs.get("base"):
            counts["nested-base" if nested else "base"] += 1
        elif name == "enumeration":
            if path_tags[1:2] == ["simpleType"]:
                counts["value"] += 1
            elif "attribute" in path_tags and not nested:
                counts["inline-values"] += 1
            else:
                counts["nested-values"] += 1
        stack.append(name)

    def end(_tag):
        stack.pop()

    parser = xml.parsers.expat.ParserCreate(namespace_separator=" ")
    parser.StartElementHandler = start
    parser.EndElementHandler = end
    with open(path, "rb") as handle:
        parser.ParseFile(handle)
    return counts


def sources(root):
    """(file, key) pairs and exclusions from the generated index, checked against the disk."""
    text = (root / "docs/spec/README.md").read_text(encoding="utf8")
    listed = dict(re.findall(r"^\| `(schemas/[^`]+)` \| `([^`]+)` \|$", text, re.M))
    tail = text.split("Excluded on purpose", 1)
    excluded = set(re.findall(r"`([^`]+)`", tail[1])) if len(tail) == 2 else set()
    problems = []
    on_disk = {f"schemas/{d}/{p.name}" for d in ("ooxml", "opc") for p in (root / "schemas" / d).glob("*.xsd")} - excluded
    index = root / "schemas/ms-extensions/index.json"
    if index.exists():
        for fname, uri, *_ in json.loads(index.read_text(encoding="utf8")):
            short = uri.split("schemas.microsoft.com/")[1].removeprefix("office/")
            if short not in excluded:
                on_disk.add(f"schemas/ms-extensions/{fname}")
    for missing in sorted(on_disk - set(listed)):
        problems.append(f"{missing}: on disk, neither a checklist source nor excluded")
    for gone in sorted(set(listed) - on_disk):
        problems.append(f"{gone}: a checklist source, but not on disk or in the index")
    return listed, problems


def row_key(row_id, keys):
    """The source key a row id starts with. Extension keys contain `/` and `:`, hence longest first."""
    for key in keys:
        if row_id.startswith(key + ":"):
            return key
    return None


def checklist_counts(path, keys):
    per_key, stray = {}, []
    ordered = sorted(keys, key=len, reverse=True)
    with open(path, encoding="utf8", newline="") as handle:
        rows = csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE)
        for row in rows:
            if row["id"].startswith(HAND_WRITTEN_PREFIX):
                continue
            key = row_key(row["id"], ordered)
            if key is None:
                stray.append(row["id"])
                continue
            c = per_key.setdefault(key, Counter())
            c[row["kind"]] += 1
            if row["kind"] == "attribute" and row["type"] == "(inline)" and row["restriction"] and "=" not in row["restriction"]:
                c["inline-values"] += row["restriction"].count(" | ") + 1
    return per_key, stray


def compare(root, checklist):
    """Return (totals by kind for schema, for checklist, list of problems)."""
    listed, problems = sources(root)
    per_key, stray = checklist_counts(checklist, set(listed.values()))
    problems += [f"{r}: row belongs to no source" for r in stray[:20]]
    raw_total, row_total = Counter(), Counter()
    for fname, key in sorted(listed.items()):
        if not (root / fname).exists():
            continue
        raw = count_schema(root / fname)
        rows = per_key.get(key, Counter())
        raw_total.update(raw)
        row_total.update(rows)
        for kind in KINDS + ("inline-values",):
            if raw[kind] != rows[kind]:
                problems.append(f"{fname} [{key}] {kind}: schema {raw[kind]}, checklist {rows[kind]}")
        for kind in sorted(k for k in raw if k.startswith("nested-")):
            problems.append(f"{fname} [{key}] {kind}: schema declares {raw[kind]} inside a nested element, checklist has no rows for them")
    return raw_total, row_total, problems


def report(raw_total, row_total, problems, out=sys.stdout):
    print("| Construct | Schemas | Checklist |", file=out)
    print("| --- | ---: | ---: |", file=out)
    for kind in KINDS + ("inline-values",):
        print(f"| {kind} | {raw_total[kind]} | {row_total[kind]} |", file=out)
    for kind in sorted(k for k in raw_total if k.startswith("nested-")):
        print(f"| {kind} | {raw_total[kind]} | no rows |", file=out)
    for line in problems:
        print(f"MISMATCH {line}", file=out)
    print(f"{sum(row_total[k] for k in KINDS)} rows; {len(problems)} mismatches", file=out)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2])
    ap.add_argument("--checklist", type=pathlib.Path)
    args = ap.parse_args(argv)
    raw, rows, problems = compare(args.root, args.checklist or args.root / "docs/spec/checklist.tsv")
    report(raw, rows, problems)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
