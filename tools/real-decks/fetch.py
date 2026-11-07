#!/usr/bin/env python3
"""F-007: fetch every deck in fixtures/real/README.md's provenance table, for local tests.

    python3 -I tools/real-decks/fetch.py NEW_EMPTY_DIR

Saves under the table's names, never the server's, and deletes and reports a file whose
SHA-256 differs from the table. Decks are untrusted: use a new empty directory, keep no
scripts there, and read them only with tools that bound them.
"""

import hashlib
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
ROW = re.compile(r"^\| `([\w.-]+)` \| [^|]* \| \[source\]\(([^)\s]+)\) \|.*?\| `([0-9a-f]{64})` \|")


def main(argv):
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    out = pathlib.Path(argv[1])
    out.mkdir(parents=True, exist_ok=True)
    if any(out.iterdir()):
        print(f"{out} is not empty", file=sys.stderr)
        return 2
    failed = 0
    table = (ROOT / "fixtures/real/README.md").read_text(encoding="utf8")
    rows = [m.groups() for m in map(ROW.match, table.splitlines()) if m]
    if not rows:
        print("no rows read from the provenance table", file=sys.stderr)
        return 2
    for name, url, sha in rows:
        target = out / name
        done = subprocess.run(
            ["curl", "-sS", "-L", "--max-time", "180", "--max-filesize", "60000000",
             "--proto", "=https", "--proto-redir", "=https", "-o", str(target), url],
            capture_output=True, text=True, check=False,
        )
        if done.returncode != 0:
            print(f"FAILED {name}: {done.stderr.strip()[:120]}")
            target.unlink(missing_ok=True)
            failed += 1
        elif hashlib.sha256(target.read_bytes()).hexdigest() != sha:
            print(f"CHANGED {name}: SHA-256 differs from the table; removed")
            target.unlink()
            failed += 1
    print(f"{len(rows) - failed} of {len(rows)} decks fetched and verified")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
