#!/usr/bin/env python3
"""docs/20 registers every stable code the code holds, and only those (DOC-010).

A code a host can log with no entry saying what it means is an undocumented API, and
three whole families reached a tracker row that way. Read from the code: every
`CS-XXX-NNNN` in crates/, tests aside. Read from the page: every code it spells out, and
the short `0002` forms that follow one on a line (`CS-FND-0001 external; 0002 unsafe`).

    python3 tools/check-error-codes.py [ROOT]
"""

import pathlib
import re
import sys

DOC = "docs/20-ERROR-CODE-REGISTRY.md"
FULL = re.compile(r"\bCS-[A-Z]{3}-[0-9]{4}\b")
# A short form stands alone: `2.1.1292` is a section of another document, `2026-11` a date.
ON_LINE = re.compile(r"\bCS-([A-Z]{3})-([0-9]{4})\b|(?<![-.0-9])\b([0-9]{4})\b(?![-.][0-9])")


def in_code(root):
    found = {}
    for path in sorted((root / "crates").rglob("*.rs")):
        relative = path.relative_to(root)
        if "tests" in relative.parts or path.name == "tests.rs":
            continue
        for code in FULL.findall(path.read_text(encoding="utf8", errors="replace")):
            found.setdefault(code, relative)
    return found


def in_doc(text):
    found = set()
    for line in text.splitlines():
        family = None
        for match in ON_LINE.finditer(line):
            if match.group(1):
                family = match.group(1)
                found.add(f"CS-{family}-{match.group(2)}")
            elif family:
                found.add(f"CS-{family}-{match.group(3)}")
    return found


def check(root):
    root = pathlib.Path(root)
    code = in_code(root)
    doc = in_doc((root / DOC).read_text(encoding="utf8"))
    problems = [f"{c}: in {code[c]}, not in {DOC}" for c in sorted(set(code) - doc)]
    problems += [f"{c}: in {DOC}, in no code" for c in sorted(doc - set(code))]
    if not code:
        problems.append("no codes were found in crates/, so the reader is broken")
    return problems, f"error codes: {len(code)} in the code, each in {DOC}"


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    problems, summary = check(root)
    if not problems:
        print(summary)
    for problem in problems:
        print(problem, file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
