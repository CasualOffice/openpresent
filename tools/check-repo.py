#!/usr/bin/env python3
"""Repository gates: unique tracker ids, closed status vocabulary, a complete doc
index, working relative links, and no Rust outside the XML walker reaching for a
raw XML reader. Exits non-zero and names every failure."""
import pathlib
import re
import sys
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parents[1]
STATUS = {"Open", "In progress", "Blocked", "Done"}
errors = []

tracker = (ROOT / "docs/14-EXECUTION-TRACKER.md").read_text(encoding="utf8")
rows = re.findall(r"^\| ([A-Z]+-[0-9A-Z]+) \|[^|]*\| ([^|]*?) \|", tracker, re.M)
for rid, n in Counter(r for r, _ in rows).items():
    if n > 1:
        errors.append(f"tracker id used {n} times: {rid}")
errors += [f"{r}: status {s!r}" for r, s in rows if s not in STATUS]

index = (ROOT / "docs/00-README.md").read_text(encoding="utf8")
for doc in (ROOT / "docs").glob("*.md"):
    if doc.name != "00-README.md" and f"({doc.name})" not in index:
        errors.append(f"not in the doc index: {doc.name}")

for md in [*ROOT.glob("*.md"), *(ROOT / "docs").glob("*.md"), ROOT / "schemas/README.md", ROOT / "docs/spec/README.md"]:
    for link in re.findall(r"\]\(([^)#\s]+?)(?:#[^)]*)?\)", md.read_text(encoding="utf8")):
        if not link.startswith("http") and not (md.parent / link).exists():
            errors.append(f"{md.relative_to(ROOT)}: broken link {link}")

# docs/07: every XML part goes through one bounded walker.
for rs in (ROOT / "crates").rglob("*.rs"):
    if "quick_xml" in rs.read_text(encoding="utf8") and rs.name != "xml.rs":
        errors.append(f"{rs.relative_to(ROOT)}: uses quick_xml outside the walker")

if errors:
    sys.exit("\n".join(errors))
print(f"repository gates pass: {len(rows)} tracker rows")
