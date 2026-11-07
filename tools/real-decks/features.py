#!/usr/bin/env python3
"""F-007: what a deck exercises, read from its package. Run with `python3 -I`.

Reports the writer (docProps/app.xml), conformance class, languages, and which
constructs appear: ChartEx, Zoom, Morph, SmartArt, charts, tables, media, OLE, ink,
equations, comments, sections, 3D models, SVG, VBA, signatures, embedded fonts, and
the `buSzPct` forms (F4-02). Also every extension URI and namespace seen, so constructs
nobody listed still show.

CORP-01 also reads the producer, from evidence in the package and not only app.xml, the
PowerPoint version, the scripts its slide text uses (names and counts, never the text), and
whether a Strict deck looks saved by PowerPoint or converted by a tool (STRICT-02).

Decks are untrusted: entries are read in memory under bounds, never extracted, and a
part with a DOCTYPE is not read. Presence is found by searching the text, which is
enough to say a construct occurs and says nothing of whether it is well formed.

    python3 -I tools/real-decks/features.py --json deck.pptx ...
"""

import collections
import html
import json
import re
import sys
import zipfile

MAX_PART = 64 << 20
MAX_ENTRIES = 100_000
MAX_RATIO = 1000

NS = {
    "chartex": "http://schemas.microsoft.com/office/drawing/2014/chartex",
    "sectionzoom": "http://schemas.microsoft.com/office/powerpoint/2016/sectionzoom",
    "slidezoom": "http://schemas.microsoft.com/office/powerpoint/2016/slidezoom",
    "summaryzoom": "http://schemas.microsoft.com/office/powerpoint/2016/summaryzoom",
    "morph": "http://schemas.microsoft.com/office/powerpoint/2015/09/main",
    "math": "http://schemas.openxmlformats.org/officeDocument/2006/math",
    "model3d": "http://schemas.microsoft.com/office/drawing/2017/model3d",
    "svg": "http://schemas.microsoft.com/office/drawing/2016/SVG/main",
    "ink": "http://www.w3.org/2003/InkML",
    "modern_comments": "http://schemas.microsoft.com/office/powerpoint/2018/8/main",
    "p14": "http://schemas.microsoft.com/office/powerpoint/2010/main",
    "p15": "http://schemas.microsoft.com/office/powerpoint/2012/main",
    "strict": "http://purl.oclc.org/ooxml/presentationml/main",
}


# Letters per script, by Unicode block. Each range is the block's letters, not its digits or
# punctuation, so a stray symbol cannot make a deck look multi-script.
SCRIPTS = (
    ("Latin", 0x41, 0x5A), ("Latin", 0x61, 0x7A), ("Latin", 0xC0, 0x24F), ("Latin", 0x1E00, 0x1EFF),
    ("Greek", 0x370, 0x3FF), ("Greek", 0x1F00, 0x1FFF), ("Cyrillic", 0x400, 0x52F),
    ("Armenian", 0x531, 0x587), ("Hebrew", 0x5D0, 0x5EA), ("Arabic", 0x620, 0x6FF), ("Arabic", 0x750, 0x77F),
    ("Devanagari", 0x900, 0x97F), ("Bengali", 0x980, 0x9FF), ("Tamil", 0xB80, 0xBFF), ("Thai", 0xE01, 0xE5B),
    ("Hangul", 0x1100, 0x11FF), ("Hangul", 0xAC00, 0xD7A3), ("Kana", 0x3040, 0x30FF), ("Han", 0x3400, 0x4DBF),
    ("Han", 0x4E00, 0x9FFF), ("Han", 0x20000, 0x2FFFF),
)
# A script counts if it holds this many letters: one foreign word in a deck is not a corpus
# of that script, and a shaping or line-break defect needs running text to show.
MIN_SCRIPT_LETTERS = 20
VERSIONS = {"12": "2007", "14": "2010", "15": "2013", "16": "2016+"}


def scripts_of(text):
    """Letter count per script, most used first. Names and counts only."""
    counts = collections.Counter()
    for char, n in collections.Counter(text).items():
        code = ord(char)
        if code < 0x41:
            continue
        name = next((name for name, low, high in SCRIPTS if low <= code <= high), None)
        if name:
            counts[name] += n
    return {k: v for k, v in counts.most_common() if v >= MIN_SCRIPT_LETTERS}


def producer_of(application, alltext):
    """Who wrote the package: `(producer, evidence)`.

    app.xml alone cannot be believed: Keynote and Google Slides can name PowerPoint, and
    a deck from another tool can too. So marks only those writers leave come first.
    """
    if "Google Shape;" in alltext:
        return "Google Slides", "shape names Google writes"
    if "Body Level One" in alltext and "Title Text" in alltext:
        return "Keynote", "master placeholder text Keynote writes"
    application = application or ""
    if "Macintosh" in application:
        return "PowerPoint Mac", "app.xml"
    if re.fullmatch(r"Microsoft (Office )?PowerPoint", application):
        return "PowerPoint Windows", "app.xml"
    lowered = application.lower()
    for mark, name in (("libreoffice", "LibreOffice"), ("openoffice", "OpenOffice"), ("wps", "WPS Office"),
                       ("aspose", "Aspose"), ("python-pptx", "python-pptx"), ("canva", "Canva"),
                       ("keynote", "Keynote"), ("google", "Google Slides")):
        if mark in lowered:
            return name, "app.xml"
    return ("Other", "app.xml") if application else ("Unknown", "no application in app.xml")


def strict_form(texts):
    """`percent` if Strict namespaces carry Strict's `50%` values, `numeric` if they carry
    Transitional's `50000`, which is what a converting tool leaves behind (STRICT-02)."""
    sample = "\n".join(t for n, t in texts.items() if n.startswith(("ppt/theme/", "ppt/slideMasters/")))
    numeric = len(re.findall(r'<a:(?:lumMod|lumOff|satMod|tint|shade|alpha) val="\d+"', sample))
    percent = len(re.findall(r'<a:(?:lumMod|lumOff|satMod|tint|shade|alpha) val="[\d.]+%"', sample))
    if numeric == 0 and percent == 0:
        return "unknown"
    return "percent" if numeric == 0 else "numeric"


def text_of(archive, info):
    if info.file_size > MAX_PART:
        return None
    if info.compress_size and info.file_size // max(info.compress_size, 1) > MAX_RATIO:
        return None
    data = archive.read(info)
    head = data[:4096]
    if b"<!DOCTYPE" in head or b"<!doctype" in head:
        return None
    for enc in ("utf-8", "utf-16"):
        try:
            return data.decode(enc)
        except UnicodeDecodeError:
            continue
    return None


def scan(path):
    out = {"deck": path.rsplit("/", 1)[-1]}
    with zipfile.ZipFile(path) as archive:
        infos = [i for i in archive.infolist() if not i.is_dir()]
        if len(infos) > MAX_ENTRIES:
            return dict(out, error="too many entries")
        names = [i.filename for i in infos]
        texts = {}
        for info in infos:
            if info.filename.endswith((".xml", ".rels", ".vml")):
                t = text_of(archive, info)
                if t is not None:
                    texts[info.filename] = t
    app = texts.get("docProps/app.xml", "")
    grab = lambda tag: (re.search(rf"<{tag}>([^<]*)</{tag}>", app) or [None, ""])[1]
    application, version = grab("Application"), grab("AppVersion")
    out["writer"] = f"{application} {version}".strip()
    out["platform"] = "Mac" if "Macintosh" in application else ("Windows" if "PowerPoint" in application else "?")
    alltext = "\n".join(texts.values())
    out["strict"] = NS["strict"] in texts.get("ppt/presentation.xml", "")
    if out["strict"]:
        out["strict_form"] = strict_form(texts)
    out["application"] = application
    out["version"] = VERSIONS.get(version.split(".")[0], "") if "PowerPoint" in application else ""
    out["producer"], out["producer_evidence"] = producer_of(application, alltext)
    runs = re.findall(r"<a:t(?:\s[^>]*)?>([^<]*)</a:t>",
                      "\n".join(t for n, t in texts.items() if re.match(r"ppt/(slides|notesSlides)/[^/]+\.xml$", n)))
    out["scripts"] = scripts_of(html.unescape("\n".join(runs)))
    slides = [n for n in names if re.match(r"ppt/slides/slide\d+\.xml$", n)]
    out["slides"] = len(slides)
    langs = collections.Counter(re.findall(r'\blang="([A-Za-z]{2,3}(?:-[A-Za-z0-9]+)*)"', alltext))
    out["languages"] = [l for l, _ in langs.most_common(4)]
    bu = re.findall(r'<a:buSzPct val="([^"]*)"', alltext)
    out["buSzPct"] = {
        "numeric": sum(1 for v in bu if re.fullmatch(r"-?\d+", v)),
        "percent": sum(1 for v in bu if v.endswith("%")),
    }
    zoom = [k for k in ("sectionzoom", "slidezoom", "summaryzoom") if NS[k] in alltext]
    feats = {
        "chartex": NS["chartex"] in alltext or any("chartEx" in n for n in names),
        "zoom": ",".join(zoom),
        "morph": bool(re.search(r"<[A-Za-z0-9]+:morph\b", alltext)) and NS["morph"] in alltext,
        "smartart": any(n.startswith("ppt/diagrams/") for n in names),
        "charts": sum(1 for n in names if re.match(r"ppt/charts/chart\d+\.xml$", n)),
        "tables": alltext.count("<a:tbl>"),
        "video": sum(1 for n in names if re.search(r"\.(mp4|m4v|mov|wmv|avi|mpg)$", n, re.I)),
        "audio": sum(1 for n in names if re.search(r"\.(mp3|m4a|wav|wma)$", n, re.I)),
        "ole": sum(1 for n in names if n.startswith("ppt/embeddings/")),
        "ink": NS["ink"] in alltext or "contentPart" in alltext,
        "math": NS["math"] in alltext,
        "comments": any("comment" in n.lower() for n in names),
        "modern_comments": NS["modern_comments"] in alltext,
        "sections": "sectionLst" in alltext,
        "model3d": NS["model3d"] in alltext,
        "svg": NS["svg"] in alltext,
        "vba": any(n.lower().endswith("vbaproject.bin") for n in names),
        "signature": any(n.startswith("_xmlsignatures/") for n in names),
        "embedded_fonts": "embeddedFontLst" in texts.get("ppt/presentation.xml", ""),
        "animations": sum(1 for n in slides if "<p:timing>" in texts.get(n, "")),
        "transitions": sum(1 for n in slides if "<p:transition" in texts.get(n, "")),
        "alternate_content": alltext.count("AlternateContent>") // 2,
        "custom_xml": any(n.startswith("customXml/") for n in names),
        "notes": sum(1 for n in names if re.match(r"ppt/notesSlides/notesSlide\d+\.xml$", n)),
    }
    out["features"] = {k: v for k, v in feats.items() if v}
    vt = re.findall(r"<vt:array\b[^>]*>", texts.get("docProps/custom.xml", ""))
    if vt:
        out["vt_array"] = sorted(set(re.findall(r"\b([lu]Bounds?)=", " ".join(vt))))
    sz = re.search(r'serverZoom="([^"]*)"', texts.get("ppt/presentation.xml", ""))
    if sz:
        out["serverZoom"] = "percent" if sz.group(1).endswith("%") else "numeric"
    out["ext_uris"] = sorted(set(re.findall(r'<(?:[A-Za-z0-9]+:)?ext uri="(\{[0-9A-Fa-f-]+\})"', alltext)))
    out["namespaces"] = sorted(set(re.findall(r'xmlns:[A-Za-z0-9]+="(http://schemas\.microsoft\.com/[^"]+)"', alltext)))
    return out


def main(argv):
    as_json = "--json" in argv
    decks = [a for a in argv[1:] if a != "--json"]
    results = []
    for deck in decks:
        try:
            results.append(scan(deck))
        except (zipfile.BadZipFile, OSError, RuntimeError, NotImplementedError, EOFError) as err:
            results.append({"deck": deck.rsplit("/", 1)[-1], "error": type(err).__name__})
    if as_json:
        print(json.dumps(results, indent=1, ensure_ascii=False))
    else:
        for r in results:
            print(r["deck"], r.get("writer"), r.get("slides"), r.get("features"))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
