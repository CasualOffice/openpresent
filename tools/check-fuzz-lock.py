#!/usr/bin/env python3
"""Every separate cargo workspace's lockfile agrees with its manifests.

Ported from OpenCalc's `tools/check-fuzz-lock.py` at `b23c67c`. `fuzz/` is its own
workspace with its own lockfile, and it path-depends on the engine crates, so a
dependency change in crates/ can leave `fuzz/Cargo.lock` stale while the main build's
`--locked` stays green, because that is not the lockfile it reads. OpenCalc paid for
this twice: an audit that never read the fuzz lockfile missed a vulnerable `h2`, and a
stale fuzz lockfile turned main red, reported as a registry outage.

`cargo metadata --locked` refuses when the lockfile would have to change, which answers
the question in seconds instead of in a fuzz build.

Workspaces are found, not listed: every tracked `Cargo.lock` other than the root's. A
hand-kept list is how OpenCalc's `desktop/` went unchecked and red.

    python3 tools/check-fuzz-lock.py [ROOT]
"""

import pathlib
import subprocess
import sys


def workspaces(root):
    listed = subprocess.run(
        ["git", "ls-files", "--", "*Cargo.lock"], cwd=root, capture_output=True, text=True, check=False,
    ).stdout.split()
    if not listed:
        listed = [str(p.relative_to(root)) for p in root.rglob("Cargo.lock") if "target" not in p.parts]
    return sorted(
        root / pathlib.Path(lock).parent / "Cargo.toml"
        for lock in listed
        if pathlib.Path(lock).parent != pathlib.Path(".")
    )


def check(root):
    root = pathlib.Path(root).resolve()
    problems = []
    found = workspaces(root)
    for manifest in found:
        rel = manifest.parent.relative_to(root)
        if not manifest.is_file():
            problems.append(f"{rel}/Cargo.lock has no Cargo.toml beside it")
            continue
        done = subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", str(manifest)],
            cwd=root, capture_output=True, text=True, check=False,
        )
        if done.returncode != 0:
            detail = "; ".join(done.stderr.strip().splitlines()[:2])
            problems.append(
                f"{rel}/Cargo.lock is out of date with its manifest or path dependencies ({detail}). "
                f"Run `cargo update --manifest-path {rel}/Cargo.toml --workspace` and commit the "
                f"lockfile with the change that moved the dependency"
            )
    if not any(m.parent.name == "fuzz" for m in found) and (root / "fuzz/Cargo.toml").is_file():
        problems.append("fuzz/ has a manifest and no tracked lockfile")
    return problems, f"lockfiles: {len(found)} separate workspaces agree with their manifests"


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    problems, summary = check(root)
    if problems:
        print("stale lockfiles in separate workspaces:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
