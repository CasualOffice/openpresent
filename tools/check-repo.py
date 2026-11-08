#!/usr/bin/env python3
"""Every repository gate, in one command (docs/15).

Its own checks: unique tracker ids, one tracker and one roadmap, a complete doc index,
working relative links, and no Rust outside the XML walker reaching for a raw XML
reader. Then each `tools/check-*.py` gate listed in GATES, which docs/15 describes.
Exits non-zero and names every failure. The pre-commit hook and CI's `checks` job run
this; `check-gate-selftest.py` proves each gate can fail.
"""
import importlib.util
import pathlib
import re
import subprocess
import sys
from collections import Counter

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))
from tracker_rows import ID_ROW, TRACKER, rows  # noqa: E402

# Each takes the repository root and returns (problems, summary). check-commits judges
# history, not the tree, so it runs from the commit-msg hook and CI instead.
GATES = [
    "check-tracker-shape.py",
    "check-doc-references.py",
    "check-crate-deps.py",
    "check-fuzz-lock.py",
    "check-parser-limits.py",
    "check-error-codes.py",
    "check-workflows.py",
    "check-chart-codegen.py",
]


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_")[:-3], HERE / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def own_checks(root):
    errors = []
    tracker = (root / TRACKER).read_text(encoding="utf8")
    found = list(rows(tracker))
    # Every line that starts like a row must parse as one.
    for line in tracker.splitlines():
        if re.match(r"^\| [A-Z0-9]+-", line) and not ID_ROW.match(line):
            errors.append(f"unreadable tracker row: {line[:50]}")
    for rid, n in Counter(rid for _, rid, _ in found).items():
        if n > 1:
            errors.append(f"tracker id used {n} times: {rid}")

    index = (root / "docs/00-README.md").read_text(encoding="utf8")
    # Tracked files only: a document another session is still writing is not yet in the repo.
    tracked = subprocess.run(["git", "ls-files", "docs/*.md"], cwd=root, capture_output=True, text=True).stdout.split()
    for doc in (pathlib.Path(t) for t in tracked):
        if doc.parent.name == "docs" and doc.name != "00-README.md" and f"({doc.name})" not in index:
            errors.append(f"not in the doc index: {doc.name}")

    linked = [*root.glob("*.md"), *(root / "docs").glob("*.md")]
    linked += [root / p for p in ("schemas/README.md", "docs/spec/README.md", "fixtures/real/README.md") if (root / p).is_file()]
    for md in linked:
        for link in re.findall(r"\]\(([^)#\s]+?)(?:#[^)]*)?\)", md.read_text(encoding="utf8")):
            if not link.startswith("http") and not (md.parent / link).exists():
                errors.append(f"{md.relative_to(root)}: broken link {link}")

    # CLAUDE.md: one roadmap, one tracker.
    for md in root.rglob("*.md"):
        rel = md.relative_to(root)
        # `desktop/` build output (VS-03): its own target folder, Tauri's `gen/`, and the
        # staged copy of `webapp/`. Dependencies' READMEs there need not be UTF-8.
        build_output = rel.parts[:2] in {("desktop", "target"), ("desktop", "gen"), ("desktop", "web-dist")}
        if rel.parts[0] in {"target", "fuzz"} or build_output or ".claude" in rel.parts or str(rel).startswith("crates/casual-slide-text"):
            continue
        text = md.read_text(encoding="utf8")
        if str(rel) != TRACKER and "| ID | Title | St |" in text:
            errors.append(f"{rel}: a second tracker; docs/14 is the only one")
        if re.search(r"(?i)(^|/)(todo|tasks?|roadmap|plan)[^/]*\.md$", str(rel)) and str(rel) != "docs/06-ROADMAP-AND-DELIVERY.md":
            errors.append(f"{rel}: a second plan; docs/06 is the only roadmap")

    # docs/07: every XML part goes through one bounded walker.
    for rs in (root / "crates").rglob("*.rs"):
        if "quick_xml" in rs.read_text(encoding="utf8") and rs.name != "xml.rs":
            errors.append(f"{rs.relative_to(root)}: uses quick_xml outside the walker")
    return errors, f"repository: {len(found)} tracker rows, index, links, one tracker, one walker"


def main():
    failed = False
    for name, check in [("check-repo.py", own_checks)] + [(g, load(g).check) for g in GATES]:
        problems, summary = check(ROOT)
        if problems:
            failed = True
            print(f"{name}:", file=sys.stderr)
            for p in problems:
                print(f"  {p}", file=sys.stderr)
        else:
            print(summary)
    if failed:
        sys.exit("repository gates failed")
    print(f"repository gates pass: {len(GATES) + 1} gates")


if __name__ == "__main__":
    main()
