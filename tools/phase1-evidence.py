#!/usr/bin/env python3
"""`P1X-01`: run one of Phase 1's closing checks and record what it found, for phase1-exit.py.

    python3 tools/phase1-evidence.py budget  SCRATCH   # 500-slide open, unedited save, edit
    python3 tools/phase1-evidence.py hostile SCRATCH   # every fixtures/hostile file, time-bounded
    python3 tools/phase1-evidence.py damage  SCRATCH   # DAMAGE_FULL=1 F4_REQUIRED=1 damage corpus
    python3 tools/phase1-evidence.py export  SCRATCH   # edited decks: F4, the SDK, LibreOffice

Each writes its result under its own key in tools/phase1-evidence.json, with the commit, the
date, and the machine's load, since a time measured under load is an upper bound. Failures
are recorded, never dropped: phase1-exit.py says MET only when the record shows none, for the
fixtures as they are now. SCRATCH is a folder of the caller's, emptied of what this writes.

LibreOffice runs only in the Docker image `openpresent-lo`, with no network, a read-only root
and a memory cap, never on this machine itself (owner instructions 23 and 33). A deck that
fails there is run again from its unedited input, to tell our output from LibreOffice.

Exit: 0 if the record shows no failure, 1 if it does, 2 if the check could not run.
"""

import datetime
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = pathlib.Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "tools/phase1-evidence.json"
FIXTURES = ROOT / "fixtures"
DECK_SUFFIXES = {".pptx", ".ppsx", ".potx", ".pptm", ".ppsm", ".potm"}
LO_IMAGE = "openpresent-lo"
LO_TIMEOUT = 300
CARGO_ENV = {"CARGO_BUILD_JOBS": os.environ.get("CARGO_BUILD_JOBS", "4"), "RUST_TEST_THREADS": "1"}


def digest(paths):
    """One hash over names and bytes, so phase1-exit.py can tell evidence for another corpus."""
    h = hashlib.sha256()
    for path in sorted(paths, key=lambda p: str(p.relative_to(FIXTURES))):
        h.update(str(path.relative_to(FIXTURES)).encode() + b"\0" + hashlib.sha256(path.read_bytes()).digest())
    return h.hexdigest()


def hostile_files():
    return sorted(p for p in (FIXTURES / "hostile").iterdir() if p.is_file())


def real_decks():
    return sorted(p for p in (FIXTURES / "real").iterdir() if p.suffix == ".pptx")


def export_inputs():
    decks = [p for d in ("real", "generated") for p in sorted((FIXTURES / d).iterdir()) if p.suffix in DECK_SUFFIXES]
    saveas = sorted(p for p in (FIXTURES / "rr/sa").iterdir() if p.suffix != ".tsv")
    return decks, saveas


def meta():
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    dirty = bool(subprocess.run(["git", "status", "--porcelain", "--", "crates"], cwd=ROOT,
                                capture_output=True, text=True).stdout.strip())
    return {
        "commit": commit,
        "engine_uncommitted": dirty,
        "date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%MZ"),
        "machine": f"{platform.system()} {platform.machine()}, {os.cpu_count()} cores",
        "load_before": [round(x, 1) for x in os.getloadavg()],
    }


def cargo_test(test, filter_, extra_env=None):
    env = dict(os.environ, **CARGO_ENV, **(extra_env or {}))
    done = subprocess.run(
        ["cargo", "test", "-p", "casual-slide-export", "--locked", "--test", test, "--", filter_, "--nocapture"],
        cwd=ROOT, env=env, capture_output=True, text=True, check=False,
    )
    return done.returncode, done.stdout + done.stderr


def fields(line):
    return dict(part.split("=", 1) for part in line.split() if "=" in part)


def lines_of(output, tag):
    """What follows `tag` on each line holding it; libtest may print its own text before ours."""
    return [ln[ln.index(tag) + len(tag):].strip() for ln in output.splitlines() if tag in ln]


def record(key, result):
    data = json.loads(EVIDENCE.read_text(encoding="utf8")) if EVIDENCE.exists() else {}
    data[key] = result
    EVIDENCE.write_text(json.dumps(data, indent=1, sort_keys=True, ensure_ascii=False) + "\n", encoding="utf8")
    print(json.dumps(result, indent=1, ensure_ascii=False)[:3000])
    return 0 if result.get("ok") else 1


def budget(_scratch):
    m = meta()
    code, out = cargo_test("phase1_exit", "a_500_slide_deck")
    got = lines_of(out, "PHASE1 budget")
    m["load_after"] = [round(x, 1) for x in os.getloadavg()]
    if not got:
        return record("budget", {**m, "ok": False, "exit": code, "error": out[-1500:]})
    f = fields(got[0])
    return record("budget", {
        **m, "ok": code == 0, "exit": code,
        "deck_sha256": hashlib.sha256((FIXTURES / "generated/five-hundred-slides.pptx").read_bytes()).hexdigest(),
        "optimized": f["optimized"] == "true",
        "measured_ms": {k: int(f[f"{k}_ms"]) for k in ("open", "unedited_save", "edit")},
        "budget_ms": {k: int(f[f"budget_{k}_ms"]) for k in ("open", "unedited_save", "edit")},
    })


def hostile(_scratch):
    m = meta()
    code, out = cargo_test("phase1_exit", "every_hostile_fixture")
    got = lines_of(out, "PHASE1 hostile ")
    m["load_after"] = [round(x, 1) for x in os.getloadavg()]
    files = hostile_files()
    base = {**m, "exit": code, "fixtures": len(files), "fixtures_sha256": digest(files)}
    if not got:
        # The binary died before it could say: a stack overflow or an abort is a crash.
        return record("hostile", {**base, "ok": False, "error": out[-1500:]})
    f = fields(got[0])
    refusals = sorted(lines_of(out, "PHASE1 hostile-refusal"))
    return record("hostile", {
        **base, "ok": code == 0 and f["broken"] == "0" and int(f["total"]) == len(files),
        "run": int(f["total"]), "edited": int(f["edited"]), "refused": int(f["refused"]),
        "broken": int(f["broken"]), "slowest_ms": int(f["slowest_ms"]), "bound_ms": int(f["bound_ms"]),
        "refusals": [r.split(" [")[0] for r in refusals],
    })


def damage(_scratch):
    m = meta()
    code, out = cargo_test("damage_corpus", "every_damaged_deck", {"DAMAGE_FULL": "1", "F4_REQUIRED": "1"})
    m["load_after"] = [round(x, 1) for x in os.getloadavg()]
    decks = real_decks()
    summary = {}
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("damage corpus:"):
            summary["corpus"] = line
        elif line.startswith("opened "):
            summary["opened"] = line
        elif line.startswith("F4:"):
            summary["f4"] = line
        elif line.startswith(("open ", "refuse ", "repair ")):
            summary.setdefault("counts", []).append(line)
    failure = ""
    if code != 0:
        at = out.find("panicked at")
        failure = out[at:at + 3000] if at >= 0 else out[-3000:]
    return record("damage", {
        **m, "ok": code == 0 and "f4" in summary, "exit": code,
        "decks": len(decks), "decks_sha256": digest(decks), **summary, "failure": failure,
    })


def f4(paths):
    done = subprocess.run([sys.executable, "-I", str(ROOT / "tools/f4/validate.py"), "--json", *map(str, paths)],
                          capture_output=True, text=True, check=False)
    if done.returncode not in (0, 1):
        raise SystemExit(f"F4 could not run: {done.stderr[-800:]}")
    return {d["deck"]: d for d in json.loads(done.stdout)}


def sdk(paths):
    env = dict(os.environ, OOXML_SDK_REQUIRED="1")
    done = subprocess.run([sys.executable, "-I", str(ROOT / "tools/ooxml-sdk/validate.py"), "--json", *map(str, paths)],
                          capture_output=True, text=True, check=False, env=env)
    if not done.stdout.startswith("{"):
        raise SystemExit(f"the SDK validator could not run: {done.stdout[-400:]} {done.stderr[-800:]}")
    return {d["deck"]: d for d in json.loads(done.stdout)["decks"]}


LO_SCRIPT = (
    'soffice --headless --norestore -env:UserInstallation=file:///tmp/p --convert-to pdf --outdir /tmp/o '
    '"/f/$DECK" >/tmp/log 2>&1; rc=$?; f="/tmp/o/${DECK%.*}.pdf"; '
    'if [ -s "$f" ]; then echo "PDF $(wc -c < "$f")"; fi; echo "RC $rc"; tail -c 400 /tmp/log'
)


def libreoffice(folder, name):
    """LibreOffice's reading of one deck, in a container: `pdf`, or what went wrong."""
    cmd = ["docker", "run", "--rm", "--network", "none", "--read-only", "--memory", "1500m", "--pids-limit", "256",
           "--tmpfs", "/tmp:size=600m", "--tmpfs", "/root:size=100m", "-e", "HOME=/root", "-e", f"DECK={name}",
           "-v", f"{folder}:/f:ro", LO_IMAGE, "sh", "-c", LO_SCRIPT]
    try:
        done = subprocess.run(cmd, capture_output=True, text=True, timeout=LO_TIMEOUT, check=False)
    except subprocess.TimeoutExpired:
        return {"status": "timeout"}
    rc = next((ln.split()[1] for ln in done.stdout.splitlines() if ln.startswith("RC ")), None)
    if any(ln.startswith("PDF ") for ln in done.stdout.splitlines()) and rc == "0":
        return {"status": "pdf"}
    if rc is None:
        return {"status": "docker-failed", "detail": (done.stderr or done.stdout)[-300:]}
    # 134 is an abort, 139 a segmentation fault: a crash. Anything else made no PDF.
    status = "crash" if rc in ("134", "139") or int(rc) > 128 else "no-pdf"
    return {"status": status, "rc": int(rc), "detail": done.stdout.split("\n", 2)[-1][-300:]}


def lo_ready():
    probe = subprocess.run(["docker", "image", "inspect", LO_IMAGE], capture_output=True, text=True, check=False)
    return probe.returncode == 0


def judged_f4(result):
    if result is None:
        return "missing"
    return result["status"]


def export(scratch):
    scratch = pathlib.Path(scratch).resolve()
    edited_dir = scratch / "edited"
    shutil.rmtree(edited_dir, ignore_errors=True)
    edited_dir.mkdir(parents=True)
    if not lo_ready():
        print(f"the Docker image {LO_IMAGE} is not there, so LibreOffice cannot run", file=sys.stderr)
        return 2
    m = meta()
    code, out = cargo_test("phase1_exit", "every_corpus_deck", {"PHASE1_EXPORT_OUT": str(edited_dir)})
    decks, saveas = export_inputs()
    base = {**m, "decks": len(decks), "saveas": len(saveas), "inputs_sha256": digest(decks + saveas)}
    written = lines_of(out, "PHASE1 export-write")
    if code != 0 or not written:
        return record("export", {**base, "ok": False, "exit": code,
                                 "failures": lines_of(out, "PHASE1 export-failure") or [out[-1500:]]})
    left_out = lines_of(out, "PHASE1 export-left-out")
    outputs = sorted(edited_dir.iterdir())
    packages = outputs + [p for p in saveas if p.suffix in DECK_SUFFIXES]
    others = [p for p in saveas if p.suffix not in DECK_SUFFIXES]
    source_of = {p: next(d for d in decks if d.name == p.name) for p in outputs}
    source_of.update({p: p for p in saveas})
    label = {p: (f"edited/{p.name}" if p.parent == edited_dir else f"rr/sa/{p.name}") for p in packages + others}

    failures = []
    f4_out, f4_in = f4(packages), f4(sorted({source_of[p] for p in packages}))
    for p in packages:
        status = judged_f4(f4_out.get(p.name))
        if status != "pass":
            was = judged_f4(f4_in.get(source_of[p].name))
            failures.append(f"F4 {label[p]}: {status}, input {was}")
    sdk_out = sdk(packages)
    sdk_in = sdk(sorted({source_of[p] for p in packages}))
    for p in packages:
        got = (sdk_out.get(p.name) or {}).get("status", "missing")
        if got != "pass":
            was = (sdk_in.get(source_of[p].name) or {}).get("status", "missing")
            failures.append(f"SDK {label[p]}: {got}, input {was}")

    lo = {}
    with ThreadPoolExecutor(max_workers=3) as pool:
        futures = {p: pool.submit(libreoffice, p.parent, p.name) for p in packages + others}
        for p, fut in futures.items():
            lo[p] = fut.result()
    lo_counts = {}
    # The add-in, the theme and the ODF stub are identity copies of grid inputs, not decks:
    # LibreOffice opens none of the three formats, so what it said is kept beside the rest.
    not_decks = {label[p]: lo.pop(p)["status"] for p in others}
    for p, got in lo.items():
        lo_counts[got["status"]] = lo_counts.get(got["status"], 0) + 1
        if got["status"] != "pdf":
            source = source_of[p]
            was = libreoffice(source.parent, source.name)["status"] if source != p else "is the input"
            failures.append(f"LibreOffice {label[p]}: {got['status']} {got.get('rc', '')}, unedited input {was}".strip())
    m_after = [round(x, 1) for x in os.getloadavg()]
    return record("export", {
        **base, "load_after": m_after, "exit": code, "ok": not failures,
        "written": int(fields(written[0])["total"]), "edits_left_out": left_out[0].split() if left_out else [],
        "f4_and_sdk_checked": len(packages), "libreoffice_checked": len(lo), "libreoffice": lo_counts,
        "libreoffice_not_decks": not_decks,
        "libreoffice_image": LO_IMAGE, "failures": failures,
    })


def main(argv):
    checks = {"budget": budget, "hostile": hostile, "damage": damage, "export": export}
    if len(argv) != 3 or argv[1] not in checks:
        print(__doc__, file=sys.stderr)
        return 2
    return checks[argv[1]](argv[2])


if __name__ == "__main__":
    sys.exit(main(sys.argv))
