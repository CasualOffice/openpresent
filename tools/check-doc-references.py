#!/usr/bin/env python3
"""Every document, ADR and tracker row a document cites exists.

Ported from OpenCalc's `tools/check-doc-references.py` at `b23c67c`. A dead reference
is worse than none, because it reads as a decision recorded somewhere. `DOC-008` found
two here: `docs/21` and `docs/15` were cited by the package crate and by design notes,
and neither existed.

Rules:

  1. `docs/NN` resolves to a file in docs/, or to a tombstone row in docs/00 (a row
     whose first cell is the number), which says where the content went.
  2. A spelled-out `docs/NN-NAME.md` names a file that exists. A rename leaves the
     number resolving and the file name dead, which rule 1 cannot see.
  3. `ADR-NNN` is a row in docs/08. `its ADR-018` and `OpenCalc's ADR-018` cite the
     sister repo's register, and are not held to ours.
  4. An id is defined by a row: in docs/14, or in the table of another document that
     owns that id space (`ENT-01` in docs/10, `BEH-01` in docs/51). Only ids whose
     prefix docs/14 uses are held to this, and the prefixes are derived from docs/14,
     never listed here.

**Known dead references are a ratchet, not an exemption.** `doc-references.known`
lists the dead references that existed when this gate arrived, each with the tracker
row that owns fixing it. A new dead reference fails; so does a listed one that has been
fixed, so the list can only shrink.

Not checked: whether a citation is apt. A row that exists and is the wrong row is a
question for review.

    python3 tools/check-doc-references.py [ROOT]
"""

import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from tracker_rows import TRACKER, rows  # noqa: E402

# Code copied from OpenCalc cites OpenCalc's documents, as its PROVENANCE.md says. It is
# held to rule 2 nowhere, since those names are OpenCalc's.
FOREIGN = ("crates/casual-slide-text/",)
# The self-test seeds dead citations on purpose, in order to see this gate fail on them.
SEEDS = "check-gate-selftest.py"

# An id named on purpose that is not a row on purpose, with the reason. Same idiom as
# OpenCalc's NAMED_BUT_UNDEFINED: an exemption is a decision on the record.
NAMED_BUT_UNDEFINED = {}

# `OpenCalc's docs/95` and `its docs/21` cite the sister repo, and are not held to ours.
DOC_CITE = re.compile(r"(?<!its )(?<!OpenCalc's )(?<!OpenCalc )\bdocs/([0-9]{2}[a-z]?)\b(?!-)")
DOC_FILE_CITE = re.compile(r"\bdocs/([0-9]{2}[a-z]?-[A-Z0-9][A-Za-z0-9-]*\.md)\b")
ADR_CITE = re.compile(r"(?<!its )(?<!OpenCalc's )\bADR-([0-9]{3})\b")
KNOWN = "tools/doc-references.known"
TOMBSTONE = re.compile(r"^\|\s*([0-9]{2}[a-z]?)\s*\|")


def sources(root):
    """Every file a citation may live in: documents, and the engine's own comments."""
    found = sorted((root / "docs").glob("*.md"))
    found += sorted(p for p in root.glob("*.md"))
    for extra in ("schemas/README.md", "docs/spec/README.md", "fixtures/real/README.md"):
        if (root / extra).is_file():
            found.append(root / extra)
    found += sorted(
        p for p in (root / "crates").rglob("*.rs")
        if not str(p.relative_to(root)).startswith(FOREIGN)
    ) if (root / "crates").is_dir() else []
    found += sorted(p for p in (root / "tools").rglob("*.py") if p.name != SEEDS) if (root / "tools").is_dir() else []
    return found


def known_numbers(root):
    numbers = set()
    for doc in (root / "docs").glob("*.md"):
        match = re.match(r"^([0-9]{2}[a-z]?)-", doc.name)
        if match:
            numbers.add(match.group(1))
    index = root / "docs/00-README.md"
    if index.is_file():
        for line in index.read_text(encoding="utf8").splitlines():
            row = TOMBSTONE.match(line)
            if row:
                numbers.add(row.group(1))
    return numbers


def known_adrs(root):
    register = root / "docs/08-ADR-REGISTER.md"
    return {
        line.split("|")[1].strip()
        for line in register.read_text(encoding="utf8").splitlines()
        if line.startswith("| ADR-")
    }


def check(root):
    root = pathlib.Path(root)
    for required in ("docs/00-README.md", "docs/08-ADR-REGISTER.md", TRACKER):
        if not (root / required).is_file():
            return [f"{required} is missing"], ""
    numbers = known_numbers(root)
    adrs = known_adrs(root)
    tracked = {rid for _, rid, _ in rows((root / TRACKER).read_text(encoding="utf8"))}
    ids = set(tracked)
    for doc in (root / "docs").glob("*.md"):
        ids.update(rid for _, rid, _ in rows(doc.read_text(encoding="utf8")))
    if not adrs or not tracked:
        return ["the ADR register or the tracker parsed as empty"], ""

    # Numbered ids only: `T-1A` and `P-3` are tier and phase rows, cited by name.
    prefixes = {i.rsplit("-", 1)[0] for i in tracked if re.search(r"-[0-9]{2,3}$", i)} - {"ADR"}
    id_cite = re.compile(
        r"(?<![A-Za-z0-9-])((?:" + "|".join(sorted(map(re.escape, prefixes), key=len, reverse=True))
        + r")-[0-9]{2,3})(?![A-Za-z0-9-])"
    )

    found = []
    files = sources(root)
    for path in files:
        rel = path.relative_to(root)
        text = path.read_text(encoding="utf8", errors="replace")
        for number in sorted(set(DOC_CITE.findall(text))):
            if number not in numbers:
                found.append((str(rel), f"docs/{number}"))
        for name in sorted(set(DOC_FILE_CITE.findall(text))):
            if not (root / "docs" / name).is_file():
                found.append((str(rel), f"docs/{name}"))
        for number in sorted(set(ADR_CITE.findall(text))):
            if f"ADR-{number}" not in adrs and f"ADR-{number}" not in ids:
                found.append((str(rel), f"ADR-{number}"))
        for identifier in sorted(set(id_cite.findall(text))):
            if identifier not in ids and identifier not in NAMED_BUT_UNDEFINED:
                found.append((str(rel), identifier))

    known = {}
    known_file = root / KNOWN
    if known_file.is_file():
        for line in known_file.read_text(encoding="utf8").splitlines():
            if line.strip() and not line.startswith("#"):
                path, cited, owner = line.split("\t")[:3]
                known[(path, cited)] = owner
    problems = []
    for path, cited in found:
        if (path, cited) not in known:
            problems.append(f"{path}: cites {cited}, which does not exist")
    for (path, cited), owner in sorted(known.items()):
        if owner not in tracked:
            problems.append(f"{KNOWN}: {path} {cited} names {owner}, which is not a row in docs/14")
        if (path, cited) not in found:
            problems.append(f"{KNOWN}: {path} no longer cites a dead {cited}; delete the line")
    stale = [i for i in NAMED_BUT_UNDEFINED if i in ids]
    problems += [f"exemption {i} is no longer needed: it is now a row" for i in stale]
    return problems, (
        f"doc references: {len(files)} files, {len(numbers)} doc numbers, {len(adrs)} ADRs, "
        f"{len(ids)} ids; every citation resolves, {len(known)} known dead references owned by rows"
    )


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    problems, summary = check(root)
    if problems:
        print("citations of things that do not exist:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        print(
            "\nRenamed: correct the citation. Retired: add a tombstone row to docs/00. Never\n"
            "existed: the citation is the defect. Deliberately named: add it to\n"
            "NAMED_BUT_UNDEFINED here, with the reason.",
            file=sys.stderr,
        )
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
