#!/usr/bin/env python3
"""docs/21 lists every limit the code holds, with the default the code holds.

A limits page written from memory drifts the first time someone tightens a cap, and a
reviewer reading the page then trusts a number the code does not have. So the page is
held to the code, in one direction that matters: everything in the code is on the
page, quoted as the code writes it.

Read from the code:

  * every field of a `*Limits` struct in crates/ (other than the copied `-text`), with
    its value in that struct's `impl Default`;
  * every `const MAX_*` in crates/.

Each must appear in docs/21 as a table row starting `` | `name` | `default` | ``. A
field with no default in its `Default` impl is itself a finding, since the page could
not say what it is.

    python3 tools/check-parser-limits.py [ROOT]
"""

import pathlib
import re
import sys

DOC = "docs/21-PARSER-LIMITS.md"
SKIP = ("crates/casual-slide-text/",)
STRUCT = re.compile(r"pub struct (\w*Limits)\s*\{(.*?)\n\}", re.S)
FIELD = re.compile(r"^\s*pub (\w+)\s*:", re.M)
CONST = re.compile(r"\bconst (MAX_\w+)\s*:\s*[\w:]+\s*=\s*([^;]+);")


def default_values(text, struct):
    """`name: expr,` pairs in `impl Default for <struct>`."""
    match = re.search(r"impl Default for " + struct + r"\s*\{.*?Self\s*\{(.*?)\n\s*\}\s*\n\s*\}", text, re.S)
    if not match:
        return {}
    values = {}
    for line in match.group(1).splitlines():
        line = line.split("//")[0].strip().rstrip(",")
        if ":" in line:
            name, value = line.split(":", 1)
            values[name.strip()] = value.strip()
    return values


def code_limits(root):
    """(name, default or None, where) for every limit in the code."""
    found = []
    for path in sorted((root / "crates").rglob("*.rs")):
        rel = str(path.relative_to(root))
        if rel.startswith(SKIP) or "/tests/" in rel or rel.endswith("tests.rs"):
            continue
        text = path.read_text(encoding="utf8")
        for struct, body in STRUCT.findall(text):
            values = default_values(text, struct)
            for field in FIELD.findall(body):
                found.append((field, values.get(field), f"{rel}: {struct}"))
        for name, value in CONST.findall(text):
            found.append((name, " ".join(value.split()), rel))
    return found


def check(root):
    root = pathlib.Path(root)
    doc = root / DOC
    if not doc.is_file():
        return [f"{DOC} is missing"], ""
    rows = {}
    for line in doc.read_text(encoding="utf8").splitlines():
        match = re.match(r"^\| `([A-Za-z_]+)` \| `([^`]+)` \|", line)
        if match:
            rows.setdefault(match.group(1), set()).add(match.group(2))
    problems = []
    limits = code_limits(root)
    for name, value, where in limits:
        if value is not None and re.fullmatch(r"\w+Limits::default\(\)", value):
            continue  # a nested limits struct, such as `OoxmlLimits::package`, listed field by field
        if value is None:
            problems.append(f"{where}: {name} has no value in its Default impl")
            continue
        if name not in rows:
            problems.append(f"{where}: {name} = {value} is not in {DOC}")
        elif value not in rows[name]:
            problems.append(f"{where}: {name} is {value} in the code and {sorted(rows[name])} in {DOC}")
    if not limits:
        problems.append("no limits were found in crates/, so the reader is broken")
    return problems, f"parser limits: {len(limits)} limits in the code, each in {DOC} with its default"


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    problems, summary = check(root)
    if problems:
        print(f"limits the code holds and {DOC} does not say:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
