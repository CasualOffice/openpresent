#!/usr/bin/env python3
"""The workflows keep docs/15's contract.

  1. **Actions are pinned by full commit SHA.** A tag can be moved to code nobody
     reviewed; a SHA cannot. Local actions (`./...`) are ours and exempt.
  2. **No retries** (docs/15, flake policy). A retry turns an intermittent failure, the
     kind most worth knowing about, into a pass. Retry actions and options, and
     `continue-on-error: true`, are refused outside comments.
  3. **Every workflow declares `permissions:`**, so none runs with the default token.
  4. **The job list is a contract in both directions**, as OpenCalc's docs/15 learned:
     every job a workflow runs is a row in docs/15's job table, and every row is a job.
     Three jobs once reached OpenCalc's CI and not its table.

Parsed by indentation, with the standard library alone, so the gate runs wherever
python3 does.

    python3 tools/check-workflows.py [ROOT]
"""

import pathlib
import re
import sys

DOC = "docs/15-CI-AND-RELEASE-GATES.md"
USES = re.compile(r"^\s*(?:-\s*)?uses:\s*([^\s#]+)")
PINNED = re.compile(r"^[\w.-]+/[\w./-]+@[0-9a-f]{40}$")
RETRY = re.compile(r"(?i)(\bretr(?:y|ies)\b|continue-on-error:\s*true|max[_-]?attempts|\brerun\b)")
JOB_ROW = re.compile(r"^\| `([a-z0-9-]+)` \| `([a-z0-9-]+\.yml)` \|")


def jobs_of(text):
    """Job ids: two-space keys under a top-level `jobs:`."""
    found = []
    inside = False
    for line in text.splitlines():
        if re.match(r"^jobs:\s*$", line):
            inside = True
            continue
        if inside and re.match(r"^\S", line):
            inside = False
        if inside:
            match = re.match(r"^  ([A-Za-z0-9_-]+):\s*(#.*)?$", line)
            if match:
                found.append(match.group(1))
    return found


def release_problems(text):
    """docs/15: a release fires on a component tag and nothing else, and a manual run is
    a dry run unless someone says otherwise."""
    found = []
    trigger = re.search(r"(?ms)^on:\s*\n(.*?)(?=^\S)", text)
    block = trigger.group(1) if trigger else ""
    for event in re.findall(r"(?m)^  ([a-z_]+):", block):
        if event not in ("push", "workflow_dispatch"):
            found.append(f"a release may not run on `{event}`")
    push = re.search(r"(?ms)^  push:\s*\n(.*?)(?=^  \S|\Z)", block)
    if push:
        if re.search(r"(?m)^    branches", push.group(1)):
            found.append("a release may not run on a branch push; tags only")
        tags = re.findall(r"[\"']?([a-z]+[-_]v\*)[\"']?", push.group(1))
        if not tags:
            found.append("a release must name component-scoped tags, such as `engine-v*`")
    if "workflow_dispatch" in block:
        dry = re.search(r"(?ms)dry[_-]run:\s*\n(.*?)(?=^      \S|\Z)", block)
        if not dry or not re.search(r"(?m)default:\s*true\s*$", dry.group(1)):
            found.append("a manual release run needs a `dry_run` input defaulting to true")
    return found


def check(root):
    root = pathlib.Path(root)
    workflows = sorted((root / ".github/workflows").glob("*.yml"))
    problems = []
    if not workflows:
        return ["no workflows found"], ""
    running = set()
    for wf in workflows:
        text = wf.read_text(encoding="utf8")
        rel = wf.relative_to(root)
        if not re.search(r"(?m)^permissions:", text):
            problems.append(f"{rel}: no top-level `permissions:`")
        for number, line in enumerate(text.splitlines(), 1):
            code = line.split(" #")[0] if not line.lstrip().startswith("#") else ""
            match = USES.match(code)
            if match and not match.group(1).startswith(("./", "docker://")) and not PINNED.match(match.group(1)):
                problems.append(f"{rel}:{number}: {match.group(1)} is not pinned to a full commit SHA")
            if code and RETRY.search(code):
                problems.append(f"{rel}:{number}: a retry, which docs/15 forbids: {code.strip()[:70]}")
        running.update((job, wf.name) for job in jobs_of(text))
        if wf.name.startswith("release"):
            problems += [f"{rel}: {p}" for p in release_problems(text)]

    doc = root / DOC
    if not doc.is_file():
        return problems + [f"{DOC} is missing, so the job contract cannot be checked"], ""
    listed = set()
    for line in doc.read_text(encoding="utf8").splitlines():
        match = JOB_ROW.match(line)
        if match:
            listed.add((match.group(1), match.group(2)))
    for job, wf in sorted(running - listed):
        problems.append(f"{wf}: job `{job}` runs and is not in {DOC}'s job table")
    for job, wf in sorted(listed - running):
        problems.append(f"{DOC}: job `{job}` in `{wf}` is in the table and does not exist")
    return problems, (
        f"workflows: {len(workflows)} files, {len(running)} jobs, all in {DOC}; actions pinned; no retries"
    )


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    problems, summary = check(root)
    if problems:
        print("workflows that break docs/15's contract:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
