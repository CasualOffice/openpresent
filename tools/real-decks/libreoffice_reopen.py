#!/usr/bin/env python3
"""`P1X-01`: LibreOffice reopens each export. **For a Linux container or CI, never a desktop.**

    OPENPRESENT_ALLOW_SOFFICE=1 python3 -I tools/real-decks/libreoffice_reopen.py EXPORTS_DIR OUT.tsv

Writes `sha256<TAB>ok|failed` per deck, which tools/phase1-exit.py reads as --libreoffice.
LibreOffice crashed on the owner's Mac (owner instruction 23, FID-04), so this refuses to
start it outside Linux, and without the variable.

Exit: 0 if all reopened, 1 if not, 2 if it was refused or could not run.
"""

import hashlib
import os
import pathlib
import subprocess
import sys
import tempfile

SECONDS = 180


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    if os.environ.get("OPENPRESENT_ALLOW_SOFFICE") != "1" or not sys.platform.startswith("linux"):
        print("refusing to start LibreOffice: Linux only, with OPENPRESENT_ALLOW_SOFFICE=1 (FID-04)", file=sys.stderr)
        return 2
    decks = sorted(p for p in pathlib.Path(argv[1]).iterdir() if p.is_file())
    if not decks:
        print(f"no decks in {argv[1]}", file=sys.stderr)
        return 2
    lines, failed = [], 0
    for deck in decks:
        with tempfile.TemporaryDirectory(prefix="reopen-") as tmp:
            try:
                done = subprocess.run(
                    ["soffice", "--headless", "--norestore", f"-env:UserInstallation=file://{tmp}/profile",
                     "--convert-to", "odp", "--outdir", tmp, str(deck)],
                    capture_output=True, check=False, timeout=SECONDS,
                )
                ok = done.returncode == 0 and any(pathlib.Path(tmp).glob("*.odp"))
            except subprocess.TimeoutExpired:
                ok = False
        failed += not ok
        lines.append(f"{hashlib.sha256(deck.read_bytes()).hexdigest()}\t{'ok' if ok else 'failed'}\t{deck.name}\n")
    pathlib.Path(argv[2]).write_text("".join(lines), encoding="utf8")
    print(f"{len(decks) - failed} of {len(decks)} reopened")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
