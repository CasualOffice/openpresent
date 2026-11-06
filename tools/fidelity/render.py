"""Turn a deck into PDF, PNG and word positions, treating the deck as hostile.

Order of defence, each independent of the next:

  1. Preflight: this Python reads the package and refuses, before LibreOffice exists,
     anything under fixtures/hostile or matching a hostile checksum, any DOCTYPE, macro,
     OLE or ActiveX part, and any non-hyperlink reference that leaves the package.
  2. A new profile per run in a new temp folder, with macros off and links not updated,
     a proxy that goes nowhere, HOME and TMPDIR moved into the temp folder, deleted after.
  3. An operating-system network denial: sandbox-exec on macOS, an empty network namespace
     on Linux. If neither works this refuses to run, unless told otherwise by the caller.
  4. Wall-clock, CPU and file-size limits, killing the whole process group.

The tools here never fetch anything, and a deck that needs the network does not render.
"""
import hashlib
import html
import json
import os
import pathlib
import posixpath
import re
import resource
import shutil
import signal
import subprocess
import sys
import tempfile
import urllib.parse
import xml.etree.ElementTree as ET
import xml.parsers.expat
import zipfile

REPO = pathlib.Path(__file__).resolve().parents[2]
SECOND_OPINION = "F7-second-opinion"
REL_NS = "{http://schemas.openxmlformats.org/package/2006/relationships}"
INERT_REL = re.compile(r"/(hyperlink)$")
ACTIVE_REL = re.compile(r"/(oleObject|control|activeXControl|activeXControlBinary|vbaProject|vbaProjectSignature|attachedTemplate)$", re.I)
SCHEME = re.compile(r"^[A-Za-z][A-Za-z0-9+.-]*:")
REMOTE_ATTR = re.compile(rb"""(?:href|src|filename|file)\s*=\s*["']\s*(?:[A-Za-z][A-Za-z0-9+.-]+:|\\\\|//)""", re.I)
LOCAL_ATTR_OK = re.compile(rb"""(?:href|src)\s*=\s*["']\s*data:""", re.I)
MAX_DECK = 256 << 20
MAX_ENTRIES = 20000
MAX_TOTAL = 1 << 30
MAX_DEPTH = 256
MAX_PIXELS = 40_000_000
SOFFICE_TIMEOUT = 180
TOOL_TIMEOUT = 120
MAX_PAGES = 500


class Refused(Exception):
    """This deck will not be given to LibreOffice, and why."""


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def hostile_checksums(fixtures):
    manifest = fixtures / "manifest.tsv"
    if not manifest.exists():
        return set()
    sums = set()
    for line in manifest.read_text(encoding="utf8").splitlines()[1:]:
        cols = line.split("\t")
        if len(cols) >= 2 and cols[0].startswith("hostile/"):
            sums.add(cols[1])
    return sums


def refuse_markup(data, part):
    """No DOCTYPE, no entity declaration, bounded depth. Expat stops at the declaration, expanding nothing."""
    depth = [0]

    def stop(what):
        def handler(*_):
            raise Refused(f"{part}: {what}")
        return handler

    def start(*_):
        depth[0] += 1
        if depth[0] > MAX_DEPTH:
            raise Refused(f"{part}: nested deeper than {MAX_DEPTH}")

    def end(*_):
        depth[0] -= 1

    parser = xml.parsers.expat.ParserCreate()
    parser.StartDoctypeDeclHandler = stop("DOCTYPE")
    parser.EntityDeclHandler = stop("entity declaration")
    parser.StartElementHandler = start
    parser.EndElementHandler = end
    try:
        parser.Parse(data, True)
    except xml.parsers.expat.ExpatError as err:
        raise Refused(f"{part}: not well formed: {err}") from err


def check_rels(data, part):
    """Relationships are how a deck reaches outside itself. Only an unfollowed hyperlink may."""
    base = posixpath.dirname(posixpath.dirname(part))
    for rel in ET.fromstring(data).iter(f"{REL_NS}Relationship"):
        typ, target = rel.get("Type", ""), rel.get("Target", "")
        mode = rel.get("TargetMode", "Internal")
        inert = bool(INERT_REL.search(typ))
        if ACTIVE_REL.search(typ):
            raise Refused(f"{part}: active content relationship {typ}")
        leaves = mode.lower() == "external" or SCHEME.match(target) or target.startswith(("\\\\", "//"))
        if leaves and not inert:
            raise Refused(f"{part}: reference outside the package: {target!r} ({typ})")
        if not leaves:
            joined = target.lstrip("/") if target.startswith("/") else posixpath.join(base, target)
            if posixpath.normpath(urllib.parse.unquote(joined)).startswith(".."):
                raise Refused(f"{part}: target above the package root: {target!r}")


def preflight(deck, fixtures=None):
    """Raises Refused, or returns what was checked. Reads every byte, so declared sizes are not trusted."""
    fixtures = fixtures or REPO / "fixtures"
    real = pathlib.Path(deck).resolve()
    hostile_dir = (fixtures / "hostile").resolve()
    if hostile_dir == real.parent or hostile_dir in real.parents:
        raise Refused(f"{deck}: under fixtures/hostile, which is never rendered")
    if not real.is_file():
        raise Refused(f"{deck}: not a file")
    if real.stat().st_size > MAX_DECK:
        raise Refused(f"{deck}: larger than {MAX_DECK} bytes")
    digest = sha256(real)
    if digest in hostile_checksums(fixtures):
        raise Refused(f"{deck}: same bytes as a fixture under fixtures/hostile")
    try:
        archive = zipfile.ZipFile(real)
    except (zipfile.BadZipFile, OSError) as err:
        raise Refused(f"{deck}: not a package: {err}") from err
    total, parts, xml_parts = 0, 0, 0
    with archive:
        infos = archive.infolist()
        if len(infos) > MAX_ENTRIES:
            raise Refused(f"{deck}: {len(infos)} entries")
        names = set()
        for info in infos:
            name = info.filename
            if name.startswith(("/", "\\")) or ".." in name.replace("\\", "/").split("/"):
                raise Refused(f"{deck}: unsafe entry name {name!r}")
            if name.lower() in names:
                raise Refused(f"{deck}: entries collide on case: {name!r}")
            names.add(name.lower())
            if info.is_dir():
                continue
            parts += 1
            low = name.lower()
            if low.endswith(("vbaproject.bin", "vbadata.xml")) or "/activex/" in low or "/embeddings/oleobject" in low:
                raise Refused(f"{deck}: active content part {name!r}")
            try:
                with archive.open(info) as handle:
                    data = handle.read(MAX_TOTAL - total + 1)
            except (zipfile.BadZipFile, OSError, RuntimeError, NotImplementedError, EOFError) as err:
                raise Refused(f"{deck}: unreadable entry {name!r}: {err}") from err
            total += len(data)
            if total > MAX_TOTAL:
                raise Refused(f"{deck}: inflates past {MAX_TOTAL} bytes")
            if low.endswith((".xml", ".rels", ".vml", ".svg")):
                xml_parts += 1
                refuse_markup(data, name)
                if low.endswith(".rels"):
                    check_rels(data, name)
                if low.endswith((".svg", ".vml")):
                    for hit in REMOTE_ATTR.finditer(data):
                        if not LOCAL_ATTR_OK.match(data, hit.start()):
                            raise Refused(f"{deck}: {name} refers outside the package")
    return {"sha256": digest, "parts": parts, "xml_parts": xml_parts, "inflated_bytes": total}


# ---------------------------------------------------------------- isolation

def sandbox_profile(writable):
    """Everything readable, no network, writes only to temp. A hostile deck should not be able to phone home or persist."""
    paths = "".join(f'(allow file-write* (subpath "{p}"))' for p in writable)
    return ("(version 1)(allow default)"
            "(deny network*)(allow network* (local unix-socket))(allow network* (remote unix-socket))"
            '(deny file-write*)(allow file-write* (literal "/dev/null") (literal "/dev/dtracehelper") (regex #"^/dev/tty"))'
            f"{paths}")


def isolation_prefix(writable):
    """(argv prefix, label) for a process with no network, or (None, None) if the OS offers none here."""
    if sys.platform == "darwin" and shutil.which("sandbox-exec"):
        prefix = ["sandbox-exec", "-p", sandbox_profile(writable)]
        label = "macOS sandbox-exec, network denied, writes limited to temp"
    elif sys.platform.startswith("linux") and shutil.which("unshare"):
        prefix = ["unshare", "--net", "--map-root-user"]
        label = "Linux empty network namespace"
    else:
        return None, None
    try:
        probe = subprocess.run(prefix + ["/usr/bin/true" if sys.platform == "darwin" else "true"],
                               capture_output=True, timeout=15, env={"PATH": os.environ.get("PATH", "")})
    except (OSError, subprocess.SubprocessError):
        return None, None
    return (prefix, label) if probe.returncode == 0 else (None, None)


REGISTRY = """<?xml version="1.0" encoding="UTF-8"?>
<oor:items xmlns:oor="http://openoffice.org/2001/registry" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<item oor:path="/org.openoffice.Office.Common/Security/Scripting"><prop oor:name="MacroSecurityLevel" oor:op="fuse"><value>3</value></prop></item>
<item oor:path="/org.openoffice.Office.Common/Security/Scripting"><prop oor:name="DisableMacrosExecution" oor:op="fuse"><value>true</value></prop></item>
<item oor:path="/org.openoffice.Office.Common/Security/Scripting"><prop oor:name="BlockUntrustedRefererLinks" oor:op="fuse"><value>true</value></prop></item>
<item oor:path="/org.openoffice.Office.Common/Load"><prop oor:name="UpdateDocMode" oor:op="fuse"><value>0</value></prop></item>
<item oor:path="/org.openoffice.Office.Common/Misc"><prop oor:name="FirstRun" oor:op="fuse"><value>false</value></prop></item>
<item oor:path="/org.openoffice.Inet/Settings"><prop oor:name="ooInetProxyType" oor:op="fuse"><value>1</value></prop></item>
<item oor:path="/org.openoffice.Inet/Settings"><prop oor:name="ooInetHTTPProxyName" oor:op="fuse"><value>127.0.0.1</value></prop></item>
<item oor:path="/org.openoffice.Inet/Settings"><prop oor:name="ooInetHTTPProxyPort" oor:op="fuse"><value>9</value></prop></item>
<item oor:path="/org.openoffice.Inet/Settings"><prop oor:name="ooInetHTTPSProxyName" oor:op="fuse"><value>127.0.0.1</value></prop></item>
<item oor:path="/org.openoffice.Inet/Settings"><prop oor:name="ooInetHTTPSProxyPort" oor:op="fuse"><value>9</value></prop></item>
</oor:items>
"""


def limits():
    resource.setrlimit(resource.RLIMIT_CPU, (SOFFICE_TIMEOUT * 2, SOFFICE_TIMEOUT * 2))
    resource.setrlimit(resource.RLIMIT_FSIZE, (2 << 30, 2 << 30))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def run_bounded(argv, timeout, cwd=None, env=None):
    """Run with a clean environment and kill the whole group on timeout. Returns (code, stdout, stderr)."""
    proc = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.DEVNULL,
                            cwd=cwd, env=env if env is not None else {"PATH": os.environ.get("PATH", "")},
                            start_new_session=True, preexec_fn=limits)
    try:
        out, err = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.communicate()
        raise Refused(f"{argv[0]}: killed after {timeout} seconds")
    finally:
        try:
            os.killpg(proc.pid, signal.SIGKILL)  # any helper it left behind
        except (ProcessLookupError, PermissionError):
            pass
    return proc.returncode, out.decode("utf8", "replace"), err.decode("utf8", "replace")


def soffice_version():
    tool = shutil.which("soffice")
    if not tool:
        raise Refused("soffice not found")
    with tempfile.TemporaryDirectory(prefix="fidelity-version-") as tmp:
        code, out, _ = run_bounded([tool, f"-env:UserInstallation={pathlib.Path(tmp).resolve().as_uri()}", "--headless", "--version"],
                                   60, cwd=tmp, env={"PATH": os.environ.get("PATH", ""), "HOME": tmp})
    m = re.search(r"LibreOffice\s+([\d.]+)", out)
    return m.group(1) if code == 0 and m else "unknown"


def convert_to_pdf(deck, out_dir, allow_no_isolation=False):
    """LibreOffice in a throwaway profile. The deck is copied under a fixed name, so a name cannot be an option."""
    tool = shutil.which("soffice")
    if not tool:
        raise Refused("soffice not found")
    with tempfile.TemporaryDirectory(prefix="fidelity-soffice-") as raw:
        tmp = pathlib.Path(raw).resolve()
        profile = tmp / "profile" / "user"
        profile.mkdir(parents=True)
        (profile / "registrymodifications.xcu").write_text(REGISTRY, encoding="utf8")
        home, scratch, out = tmp / "home", tmp / "tmp", tmp / "out"
        for p in (home, scratch, out):
            p.mkdir()
        suffix = pathlib.Path(deck).suffix.lower()
        suffix = suffix if suffix in (".pptx", ".potx", ".ppsx", ".pptm") else ".pptx"
        input_path = tmp / f"deck{suffix}"
        shutil.copyfile(deck, input_path)
        writable = [str(tmp), "/private/var/folders", "/private/tmp"]
        prefix, label = isolation_prefix(writable)
        if prefix is None:
            if not allow_no_isolation:
                raise Refused("no way to deny this process the network here (sandbox-exec or unshare); "
                              "refusing to start LibreOffice. --allow-no-network-denial overrides, for trusted decks only")
            prefix, label = [], "NONE: preflight only"
        env = {"PATH": os.environ.get("PATH", ""), "HOME": str(home), "TMPDIR": str(scratch), "LC_ALL": "C.UTF-8",
               "SAL_USE_VCLPLUGIN": "svp", "no_proxy": "", "NO_PROXY": ""}
        argv = prefix + [tool, f"-env:UserInstallation={(tmp / 'profile').as_uri()}", "--headless", "--norestore", "--nologo",
                         "--nolockcheck", "--nodefault", "--convert-to", "pdf:impress_pdf_Export", "--outdir", str(out), str(input_path)]
        code, stdout, stderr = run_bounded(argv, SOFFICE_TIMEOUT, cwd=tmp, env=env)
        produced = out / "deck.pdf"
        if not produced.exists():
            raise Refused(f"LibreOffice wrote no PDF (exit {code}): {(stdout + stderr).strip()[:400]}")
        out_dir.mkdir(parents=True, exist_ok=True)
        target = out_dir / "deck.pdf"
        shutil.copyfile(produced, target)
    return target, label


# ---------------------------------------------------------------- PDF tools

def pdfinfo(pdf):
    code, out, err = run_bounded(["pdfinfo", str(pdf)], TOOL_TIMEOUT)
    if code:
        raise Refused(f"pdfinfo: {err.strip()[:300]}")
    info = {}
    for line in out.splitlines():
        key, sep, value = line.partition(":")
        if sep:
            info[key.strip()] = value.strip()
    return info


def pdf_fonts(pdf):
    """What was actually used. A substituted font moves every word, so a comparison must be able to say so."""
    if not shutil.which("pdffonts"):
        return []
    code, out, _ = run_bounded(["pdffonts", str(pdf)], TOOL_TIMEOUT)
    if code:
        return []
    return [line.split()[0] for line in out.splitlines()[2:] if line.strip()]


def to_pngs(pdf, out_dir, dpi):
    """pdftoppm, after checking the page size will not make an absurd bitmap."""
    info = pdfinfo(pdf)
    pages = int(info.get("Pages", "0"))
    if pages > MAX_PAGES:
        raise Refused(f"{pages} pages, limit {MAX_PAGES}")
    m = re.match(r"([\d.]+) x ([\d.]+)", info.get("Page size", ""))
    if not m or float(m.group(1)) * float(m.group(2)) * (dpi / 72) ** 2 > MAX_PIXELS:
        raise Refused(f"page size {info.get('Page size')!r} at {dpi} dpi is over the limit")
    work = out_dir / "ppm-work"
    work.mkdir(parents=True, exist_ok=True)
    code, _, err = run_bounded(["pdftoppm", "-r", str(dpi), "-png", str(pdf), str(work / "p")], TOOL_TIMEOUT * 5)
    if code:
        raise Refused(f"pdftoppm: {err.strip()[:300]}")
    made = sorted(work.glob("p-*.png"), key=lambda p: int(p.stem.split("-")[-1]))
    paths = []
    for i, p in enumerate(made, 1):
        dest = out_dir / f"page-{i:03d}.png"
        p.replace(dest)
        paths.append(dest)
    shutil.rmtree(work)
    return paths, info


WORD = re.compile(r'<word xMin="([-\d.]+)" yMin="([-\d.]+)" xMax="([-\d.]+)" yMax="([-\d.]+)">(.*?)</word>')
PAGE = re.compile(r'<page width="([\d.]+)" height="([\d.]+)">')


def word_positions(pdf):
    """Per page: size and words, from `pdftotext -bbox`. Its output carries a DOCTYPE, so it is read with
    two patterns over text it escaped itself, and is never handed to an XML parser."""
    with tempfile.TemporaryDirectory(prefix="fidelity-bbox-") as tmp:
        out = pathlib.Path(tmp) / "bbox.html"
        code, _, err = run_bounded(["pdftotext", "-bbox", str(pdf), str(out)], TOOL_TIMEOUT)
        if code:
            raise Refused(f"pdftotext: {err.strip()[:300]}")
        text = out.read_text(encoding="utf8", errors="replace")
    pages = []
    chunks = re.split(r"(?=<page width=)", text)
    for chunk in chunks[1:]:
        size = PAGE.match(chunk)
        words = [{"x0": float(a), "y0": float(b), "x1": float(c), "y1": float(d), "text": html.unescape(t)}
                 for a, b, c, d, t in WORD.findall(chunk)]
        pages.append({"width": float(size.group(1)), "height": float(size.group(2)), "words": words})
    return pages


def pdf_set(pdf, out_dir, dpi=96):
    """A PDF as PNG pages and word positions, in out_dir. Returns the common metadata."""
    pngs, info = to_pngs(pdf, out_dir, dpi)
    words = word_positions(pdf)
    (out_dir / "words.json").write_text(json.dumps(words, ensure_ascii=False, indent=0), encoding="utf8")
    return {"pages": len(pngs), "dpi": dpi, "pdf_producer": info.get("Producer", ""), "pdf_creator": info.get("Creator", ""),
            "page_size": info.get("Page size", ""), "fonts": pdf_fonts(pdf)}


def render(deck, out_dir, dpi=96, allow_no_isolation=False, fixtures=None):
    """Preflight, then LibreOffice, then PNG and words. Everything in out_dir is a second opinion."""
    out_dir = pathlib.Path(out_dir)
    checked = preflight(deck, fixtures)
    out_dir.mkdir(parents=True, exist_ok=True)
    pdf, isolation = convert_to_pdf(deck, out_dir, allow_no_isolation)
    meta = pdf_set(pdf, out_dir, dpi)
    meta.update({"producer": "libreoffice", "producer_version": soffice_version(), "tier": SECOND_OPINION,
                 "note": "LibreOffice's rendering. A second opinion, and never evidence about PowerPoint.",
                 "deck": str(deck), "deck_sha256": checked["sha256"], "preflight": checked, "isolation": isolation})
    (out_dir / "meta.json").write_text(json.dumps(meta, indent=1, sort_keys=True), encoding="utf8")
    return meta
