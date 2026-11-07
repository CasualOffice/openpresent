#!/usr/bin/env python3
"""Generate the deterministic fixtures, and fixtures/manifest.tsv.

Fixed timestamps and order, so a checksum changes only when a fixture does.

Manifest columns: path, sha256, expectation, and optionally limits. Expectation is
`refuse:<code>`, or `open` plus any `<code>` findings the deck must produce,
space-separated. Limits, comma-separated `field=value` pairs of `OoxmlLimits`, are
what the deck is opened under, where the default bound is too large for a fixture
to cross at a sensible size; without them, the defaults. The corpus test in
crates/casual-slide-ooxml/tests/corpus.rs reads this file.

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
    from generate_presets import fixtures as preset_fixtures  # imports this module, so not at the top
    yield from preset_fixtures()

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
        # PKG-02: `É` and `é` are two parts. Only ASCII letters fold (ECMA-376 Part 2, 6.2.2.3).
        "non-ascii-case-distinct-parts": (zipped(deck(slide_rels=[("image", "../media/\u00c9.png"), ("image", "../media/\u00e9.png")],
                                                      parts=[("ppt/media/\u00c9.png", png), ("ppt/media/\u00e9.png", png)])), "open"),
    }
    hostile.update(xml_hostile())
    for name, (data, expect, *limits) in hostile.items():
        yield f"hostile/{name}.bin", data, expect, *limits
    yield from property_fixtures()
    yield "generated/slides-masters-layouts.pptx", zipped(slides_parts()), "open"
    yield "generated/slides-strict.pptx", zipped([(n, d) if isinstance(d, bytes) else strict([(n, d)])[0] for n, d in slides_parts()]), "open"
    # Lazy loading is measured on this: opening it must not read 500 slides.
    yield "generated/five-hundred-slides.pptx", zipped(deck(500)), "open"
    yield from slide_hostile()
    yield "generated/drawing-every-element.pptx", zipped(drawing_deck()), "open CS-FND-0001"
    yield from drawing_hostile()
    from generate_text import fixtures as text_fixtures  # imports this module, so not at the top
    yield from text_fixtures()
    from generate_table import fixtures as table_fixtures  # imports this module, so not at the top
    yield from table_fixtures()


def main_part(xml):
    return zipped(replace(deck(), "ppt/presentation.xml", xml))


def xml_hostile():
    """XML-02, XML-03: what is not namespace-well-formed XML, refused by the walker, so
    every reader agrees. SEC-016: one deck over each bound that had no test. A bound too
    large to cross at a sensible size is lowered for its deck in the limits column."""
    root = f"<p:presentation {NS}"
    links = [("hyperlink", f"https://example.com/{i}", "External") for i in range(50)]
    return {
        "xml-repeated-attribute": (main_part(f'{root} a="1" a="2"/>'), "refuse:CS-XML-0004"),
        "xml-repeated-expanded-attribute": (main_part(f'{root} xmlns:x="urn:x" xmlns:y="urn:x" x:a="1" y:a="2"/>'), "refuse:CS-XML-0004"),
        "xml-attribute-without-value": (main_part(f"{root} hidden/>"), "refuse:CS-XML-0004"),
        "xml-attributes-run-together": (main_part(f'{root}><p:sldSz cx="1"cy="2"/></p:presentation>'), "refuse:CS-XML-0004"),
        "xml-undeclared-prefix": (main_part(f"{root}><q:x/></p:presentation>"), "refuse:CS-XML-0004"),
        "xml-nul-in-name": (main_part(f"{root}><p:sld\0Sz/></p:presentation>"), "refuse:CS-XML-0004"),
        "xml-text-outside-root": (main_part(f"{root}/>stray"), "refuse:CS-XML-0004"),
        # Found by the xml_tree fuzz target: a second mark, which quick-xml drops, so the tree lost a byte.
        "xml-second-byte-order-mark": (main_part("\ufeff\ufeff" + f"{root}/>"), "refuse:CS-XML-0004"),
        "xml-too-many-attributes": (main_part(root + "".join(f' a{i}="1"' for i in range(1025)) + "/>"), "refuse:CS-XML-0003"),
        "xml-too-many-elements": (main_part(f"{root}>" + "<a/>" * 2000 + "</p:presentation>"), "refuse:CS-XML-0001", "max_xml_elements=1000"),
        "xml-part-over-size": (main_part(f"{root}><!-- " + "padding " * 1000 + "--></p:presentation>"), "refuse:CS-PKG-0009", "max_xml_part_bytes=4096"),
        "rels-per-part": (zipped(deck(slide_rels=links)), "refuse:CS-OPC-0002", "max_part_relationships=40"),
        "rels-total": (zipped(deck(slides=4, slide_rels=links[:5])), "refuse:CS-OPC-0002", "max_total_relationships=30"),
        "content-types-flood": (zipped(deck(types_extra=[(f"/ppt/x{i}.xml", "application/xml") for i in range(40)])), "refuse:CS-OPC-0003", "max_content_types=30"),
    }


CORE_NS = ('xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" '
           'xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" '
           'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"')
EP_NS = ('xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" '
         'xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"')
CP_NS = ('xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" '
         'xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"')
FMTID = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}"
CORE_REL = f"{PKG_RELS}/metadata/core-properties"
PROP_TYPES = [("/docProps/core.xml", "application/vnd.openxmlformats-package.core-properties+xml"),
              ("/docProps/app.xml", "application/vnd.openxmlformats-officedocument.extended-properties+xml"),
              ("/docProps/custom.xml", "application/vnd.openxmlformats-officedocument.custom-properties+xml"),
              ("/ppt/presProps.xml", f"{PML}.presProps+xml"),
              ("/ppt/viewProps.xml", f"{PML}.viewProps+xml"),
              ("/ppt/tags/tag1.xml", f"{PML}.tags+xml"),
              ("/ppt/fonts/font1.fntdata", "application/x-fontdata"),
              ("/customXml/item1.xml", "application/xml"),
              ("/ppt/notesMasters/notesMaster1.xml", f"{PML}.notesMaster+xml"),
              ("/ppt/handoutMasters/handoutMaster1.xml", f"{PML}.handoutMaster+xml"),
              ("/ppt/pub.htm", "text/html")]
TAGS = f'{DECL}<p:tagLst {NS}><p:tag name="K" val="V"/></p:tagLst>'


def full_deck(presentation, core="", app="", custom="", pres_props="", view_props=""):
    """Three slides, with whichever property parts are given, and a part behind every id
    the rich presentation names. Relationship ids are fixed: see the list below."""
    entries = deck(3, presentation=presentation, types_extra=PROP_TYPES, parts=[
        ("ppt/presProps.xml", pres_props or f"{DECL}<p:presentationPr {NS}/>"),
        ("ppt/viewProps.xml", view_props or f"{DECL}<p:viewPr {NS}/>"),
        ("ppt/tags/tag1.xml", TAGS),
        ("ppt/fonts/font1.fntdata", b"not a font"),
        ("customXml/item1.xml", "<x/>"),
        ("ppt/notesMasters/notesMaster1.xml", f"{DECL}<p:notesMaster {NS}/>"),
        ("ppt/handoutMasters/handoutMaster1.xml", f"{DECL}<p:handoutMaster {NS}/>"),
        ("ppt/pub.htm", b"<html/>"),
        ("ppt/_rels/viewProps.xml.rels", rels(("slide", "slides/slide1.xml"))),
        ("ppt/_rels/presProps.xml.rels", rels(("htmlPubSaveAs", "pub.htm"))),
        ("docProps/core.xml", core or f"{DECL}<cp:coreProperties {CORE_NS}/>"),
        ("docProps/app.xml", app or f"{DECL}<Properties {EP_NS}/>"),
        ("docProps/custom.xml", custom or f"{DECL}<Properties {CP_NS}/>"),
    ])
    entries = replace(entries, "_rels/.rels", rels(("officeDocument", "ppt/presentation.xml"), (CORE_REL, "docProps/core.xml"),
                                                  ("extended-properties", "docProps/app.xml"), ("custom-properties", "docProps/custom.xml")))
    return replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),  # rId1, rId2
        ("slide", "slides/slide1.xml"), ("slide", "slides/slide2.xml"), ("slide", "slides/slide3.xml"),  # rId3-5
        ("presProps", "presProps.xml"), ("viewProps", "viewProps.xml"),  # rId6, rId7
        ("tags", "tags/tag1.xml"), ("font", "fonts/font1.fntdata"), ("customXml", "../customXml/item1.xml"),  # rId8-10
        ("notesMaster", "notesMasters/notesMaster1.xml"), ("handoutMaster", "handoutMasters/handoutMaster1.xml")))  # rId11, rId12


def pres_doc(body="", attrs=""):
    return (f'{DECL}<p:presentation {NS}{attrs}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>'
            f'<p:sldIdLst><p:sldId id="256" r:id="rId3"/><p:sldId id="257" r:id="rId4"/><p:sldId id="258" r:id="rId5"/></p:sldIdLst>'
            f'<p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>{body}</p:presentation>')


def rich_parts():
    """Every Tier 1A and 1B element at least once, with what a parser would lose: comments,
    odd spacing and quoting, prefixes other than the usual, unknown extensions."""
    presentation = (
        f"{DECL}<!-- kept -->\n<p:presentation {NS} xmlns:x=\"urn:unknown\"  serverZoom='80%'\n firstSlideNum=\"5\" showSpecialPlsOnTitleSld=\"0\" rtl=\"false\" "
        'removePersonalInfoOnSave="1" compatMode="0" strictFirstAndLastChars="1" embedTrueTypeFonts="1" saveSubsetFonts="1" '
        'autoCompressPictures="0" bookmarkIdSeed="7" conformance="transitional" x:unknown="kept">\n'
        '  <p:sldMasterIdLst>\n    <p:sldMasterId id="2147483648" r:id="rId1"/>\n  </p:sldMasterIdLst>\n'
        '  <p:notesMasterIdLst><p:notesMasterId r:id="rId11"/></p:notesMasterIdLst>\n  <p:handoutMasterIdLst><p:handoutMasterId r:id="rId12"/></p:handoutMasterIdLst>\n'
        '  <p:sldIdLst>\n    <p:sldId r:id="rId3" id="256"/>\n    <!-- between -->\n    <p:sldId id="257" r:id="rId4"><p:extLst><p:ext uri="{K}"><x:y/></p:ext></p:extLst></p:sldId>\n'
        '    <p:sldId id="258" r:id="rId5"/>\n  </p:sldIdLst>\n'
        '  <p:sldSz cx="12192000" cy="6858000" type="screen16x9"/>\n  <p:notesSz cx="6858000" cy="9144000"/>\n  <p:smartTags r:id="rId8"/>\n'
        '  <p:embeddedFontLst><p:embeddedFont><p:font typeface="Fixture Sans" panose="020B0604020202020204" pitchFamily="34" charset="0"/>'
        '<p:regular r:id="rId9"/></p:embeddedFont></p:embeddedFontLst>\n'
        '  <p:custShowLst><p:custShow name="Short &amp; sweet" id="3"><p:sldLst><p:sld r:id="rId5"/><p:sld r:id="rId3"/></p:sldLst></p:custShow></p:custShowLst>\n'
        '  <p:photoAlbum bw="1" showCaptions="0" layout="2pic" frame="frameStyle3"/>\n'
        '  <p:custDataLst><p:custData r:id="rId10"/><p:tags r:id="rId8"/></p:custDataLst>\n'
        '  <p:kinsoku lang="ja-JP" invalStChars="!)" invalEndChars="(["/>\n'
        '  <p:defaultTextStyle><a:defPPr><a:defRPr lang="en-US"/></a:defPPr></p:defaultTextStyle>\n'
        '  <p:modifyVerifier cryptProviderType="rsaAES" cryptAlgorithmClass="hash" cryptAlgorithmType="typeAny" cryptAlgorithmSid="14" spinCount="100000" saltData="AAAA" hashData="BBBB"/>\n'
        '  <p:extLst><p:ext uri="{E}"><x:kept a="1"/></p:ext></p:extLst>\n</p:presentation>\n')
    pres_props = (
        f'{DECL}<p:presentationPr {NS}>\n  <p:htmlPubPr showSpeakerNotes="0" target="_blank" title="T" r:id="rId1"><p:sldAll/></p:htmlPubPr>\n  <p:webPr showAnimation="1" imgSz="1024x768" encoding="utf-8" clr="browser"/>\n'
        '  <p:prnPr prnWhat="handouts6" clrMode="gray" hiddenSlides="1" scaleToFitPaper="1" frameSlides="1"/>\n'
        '  <p:showPr loop="1" showNarration="1" showAnimation="0" useTimings="0"><p:kiosk restart="60000"/><p:custShow id="3"/>'
        '<p:penClr><a:srgbClr val="FF0000"/></p:penClr></p:showPr>\n'
        '  <p:clrMru><a:srgbClr val="00FF00"/></p:clrMru>\n  <p:extLst><p:ext uri="{P}"/></p:extLst>\n</p:presentationPr>\n')
    common = ('<p:cViewPr varScale="1"><p:scale><a:sx n="88" d="100"/><a:sy n="89" d="100"/></p:scale>'
              '<p:origin x="-1104" y="-90"/></p:cViewPr>')
    slide_view = (f'<p:cSldViewPr snapToGrid="0" snapToObjects="1" showGuides="1">{common}'
                  '<p:guideLst><p:guide orient="horz" pos="2160"/><p:guide pos="3840"/></p:guideLst></p:cSldViewPr>')
    view_props = (
        f'{DECL}<p:viewPr {NS} lastView="sldThumbnailView" showComments="0">\n'
        '  <p:normalViewPr showOutlineIcons="0" snapVertSplitter="1" vertBarState="minimized" horzBarState="maximized" preferSingleView="1">'
        '<p:restoredLeft sz="15620" autoAdjust="0"/><p:restoredTop sz="94660"/></p:normalViewPr>\n'
        f'  <p:slideViewPr>{slide_view}</p:slideViewPr>\n'
        f'  <p:outlineViewPr>{common}<p:sldLst><p:sld r:id="rId1" collapse="1"/></p:sldLst></p:outlineViewPr>\n'
        f'  <p:notesTextViewPr>{common}</p:notesTextViewPr>\n'
        f'  <p:sorterViewPr showFormatting="0">{common}</p:sorterViewPr>\n'
        f'  <p:notesViewPr>{slide_view}</p:notesViewPr>\n'
        '  <p:gridSpacing cx="76200" cy="76200"/>\n  <p:extLst><p:ext uri="{V}"/></p:extLst>\n</p:viewPr>\n')
    core = (
        f'{DECL}<cp:coreProperties {CORE_NS} xmlns:other="urn:other">\n  <dc:title>Old &amp; title</dc:title>\n  <dc:subject>Subject</dc:subject>\n'
        '  <dc:creator>Author</dc:creator>\n  <cp:keywords xml:lang="en">one, two<cp:value xml:lang="de">drei</cp:value></cp:keywords>\n'
        '  <dc:description>Words</dc:description>\n  <cp:lastModifiedBy>Editor</cp:lastModifiedBy>\n  <cp:revision>12</cp:revision>\n'
        '  <dcterms:created xsi:type="dcterms:W3CDTF">2020-01-02T03:04:05Z</dcterms:created>\n'
        '  <dcterms:modified xsi:type="dcterms:W3CDTF">2021-02-03T04:05:06.5+05:30</dcterms:modified>\n'
        '  <cp:lastPrinted>2019-12-31T23:59:59Z</cp:lastPrinted>\n  <cp:category>Cat</cp:category>\n  <cp:contentStatus>Draft</cp:contentStatus>\n'
        '  <dc:identifier>urn:x</dc:identifier>\n  <dc:language>en-GB</dc:language>\n  <cp:version>1.2</cp:version>\n'
        '  <other:title>not the title</other:title>\n</cp:coreProperties>\n')
    app = (
        f'{DECL}<Properties {EP_NS}>\n  <TotalTime>14</TotalTime>\n  <Words>120</Words>\n  <Application>Fixture</Application>\n'
        '  <PresentationFormat>Widescreen</PresentationFormat>\n  <Paragraphs>9</Paragraphs>\n  <Slides>3</Slides>\n  <Notes>1</Notes>\n'
        '  <HiddenSlides>0</HiddenSlides>\n  <MMClips>0</MMClips>\n  <ScaleCrop>false</ScaleCrop>\n'
        '  <HeadingPairs><vt:vector size="2" baseType="variant"><vt:variant><vt:lpstr>Slide Titles</vt:lpstr></vt:variant><vt:variant><vt:i4>1</vt:i4></vt:variant></vt:vector></HeadingPairs>\n'
        '  <TitlesOfParts><vt:vector size="1" baseType="lpstr"><vt:lpstr>One</vt:lpstr></vt:vector></TitlesOfParts>\n'
        '  <Company>Co &amp; Co</Company>\n  <Manager>M</Manager>\n  <Template>T.potx</Template>\n  <Pages>0</Pages>\n  <Characters>5</Characters>\n'
        '  <Lines>2</Lines>\n  <LinksUpToDate>0</LinksUpToDate>\n  <CharactersWithSpaces>6</CharactersWithSpaces>\n  <SharedDoc>0</SharedDoc>\n'
        '  <HyperlinkBase>https://example.com/</HyperlinkBase>\n'
        '  <HLinks><vt:vector size="1" baseType="variant"><vt:variant><vt:i4>2</vt:i4></vt:variant></vt:vector></HLinks>\n'
        '  <HyperlinksChanged>0</HyperlinksChanged>\n  <DigSig><vt:blob>QUJD</vt:blob></DigSig>\n  <AppVersion>16.0000</AppVersion>\n  <DocSecurity>0</DocSecurity>\n</Properties>\n')
    values = [("lpwstr", "text"), ("i1", "-1"), ("i2", "-2"), ("i4", "-4"), ("i8", "-8"), ("int", "7"), ("ui1", "1"), ("ui2", "2"), ("ui4", "4"),
              ("ui8", "8"), ("uint", "9"), ("r4", "1.5"), ("r8", "NaN"), ("decimal", "1.25"), ("lpstr", "a"), ("bstr", "b"), ("date", "2020-01-02T03:04:05Z"),
              ("filetime", "2020-01-02T03:04:05Z"), ("bool", "true"), ("cy", "1.2500"), ("error", "0x80000000"), ("clsid", FMTID), ("blob", "QUJD"), ("oblob", "QUJD"),
              ("stream", "QUJD"), ("ostream", "QUJD"), ("storage", "QUJD"), ("ostorage", "QUJD")]
    props = [f'<property fmtid="{FMTID}" pid="{i + 2}" name="{t}"><vt:{t}>{v}</vt:{t}></property>' for i, (t, v) in enumerate(values)]
    n = len(values) + 2
    props += [f'<property fmtid="{FMTID}" pid="{n}" name="empty"><vt:empty/></property>',
              f'<property fmtid="{FMTID}" pid="{n + 1}" name="null"><vt:null/></property>',
              f'<property fmtid="{FMTID}" pid="{n + 2}" name="vstream"><vt:vstream version="{FMTID}">QUJD</vt:vstream></property>',
              f'<property fmtid="{FMTID}" pid="{n + 3}" name="vec" linkTarget="rng"><vt:vector size="2" baseType="i4"><vt:i4>1</vt:i4><vt:i4>2</vt:i4></vt:vector></property>',
              f'<property fmtid="{FMTID}" pid="{n + 4}" name="arr"><vt:array lBounds="0" uBounds="1" baseType="bool"><vt:bool>0</vt:bool><vt:bool>1</vt:bool></vt:array></property>']
    custom = f'{DECL}<Properties {CP_NS}>\n  ' + "\n  ".join(props) + '\n</Properties>\n'
    return dict(presentation=presentation, core=core, app=app, custom=custom, pres_props=pres_props, view_props=view_props)


STRICT_URIS = [("http://schemas.openxmlformats.org/presentationml/2006/main", "http://purl.oclc.org/ooxml/presentationml/main"),
               ("http://schemas.openxmlformats.org/drawingml/2006/main", "http://purl.oclc.org/ooxml/drawingml/main"),
               ("http://schemas.openxmlformats.org/officeDocument/2006/extended-properties", "http://purl.oclc.org/ooxml/officeDocument/extendedProperties"),
               ("http://schemas.openxmlformats.org/officeDocument/2006/custom-properties", "http://purl.oclc.org/ooxml/officeDocument/customProperties"),
               ("http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes", "http://purl.oclc.org/ooxml/officeDocument/docPropsVTypes"),
               ("relationships/extended-properties", "relationships/extendedProperties"),
               ("relationships/custom-properties", "relationships/customProperties"),
               ("http://schemas.openxmlformats.org/officeDocument/2006/relationships", "http://purl.oclc.org/ooxml/officeDocument/relationships")]


def strict_deck():
    out = []
    for name, data in full_deck(**rich_parts()):
        if isinstance(data, str):
            for old, new in STRICT_URIS:
                data = data.replace(old, new)
            data = data.replace('conformance="transitional"', 'conformance="strict"').replace('sz="15620"', 'sz="15.62%"')
        out.append((name, data))
    return out


def seeded(unit, count):
    """`count` copies of `unit`, broken by a comment often enough to stay under the
    expansion ratio that would otherwise refuse the file before the bound under test."""
    block = []
    for i in range(0, count, 400):
        tag = hashlib.sha256(str(i).encode()).hexdigest()
        block.append(f"<!--{tag}-->" + unit * min(400, count - i))
    return "".join(block)


def property_fixtures():
    yield "generated/rich-properties.pptx", zipped(full_deck(**rich_parts())), "open"
    yield "generated/strict-properties.pptx", zipped(strict_deck()), "open"
    # One per bound in ImportLimits. Each opens: only import counts these. The code each
    # must produce is in crates/casual-slide-import/tests/hostile_values.rs.
    def main_with(tag, unit, count):
        return dict(presentation=f"{DECL}<p:presentation {NS}><p:{tag}>" + seeded(unit, count) + f"</p:{tag}></p:presentation>")

    show = '<p:custShow name="s" id="1"><p:sldLst>' + seeded('<p:sld r:id="rId3"/>', 500_001) + "</p:sldLst></p:custShow>"
    guides = "<p:guideLst>" + seeded("<p:guide/>", 65_537) + "</p:guideLst>"
    vector = '<vt:vector size="1" baseType="i4">' + seeded("<vt:i4>1</vt:i4>", 1_000_001) + "</vt:vector>"
    long_title = ("x" * 8191 + "y") * ((16 << 20) // 8192) + hostile_noise()
    limits = {
        "slides": main_with("sldIdLst", '<p:sldId id="256" r:id="rId3"/>', 100_001),
        "masters": main_with("sldMasterIdLst", '<p:sldMasterId r:id="rId1"/>', 10_001),
        "custom-shows": main_with("custShowLst", '<p:custShow name="s" id="1"/>', 10_001),
        "show-slides": dict(presentation=f"{DECL}<p:presentation {NS}><p:custShowLst>{show}</p:custShowLst></p:presentation>"),
        "fonts": main_with("embeddedFontLst", "<p:embeddedFont/>", 4_097),
        "customer-data": main_with("custDataLst", '<p:custData r:id="rId10"/>', 100_001),
        "guides": dict(presentation=pres_doc(), view_props=f"{DECL}<p:viewPr {NS}><p:slideViewPr><p:cSldViewPr>{guides}</p:cSldViewPr></p:slideViewPr></p:viewPr>"),
        "custom-properties": dict(presentation=pres_doc(), custom=f"{DECL}<Properties {CP_NS}>" + seeded("<property/>", 100_001) + "</Properties>"),
        "keywords": dict(presentation=pres_doc(), core=f"{DECL}<cp:coreProperties {CORE_NS}><cp:keywords>" + seeded("<cp:value>k</cp:value>", 100_001) + "</cp:keywords></cp:coreProperties>"),
        "variant-items": dict(presentation=pres_doc(), custom=f'{DECL}<Properties {CP_NS}><property fmtid="{FMTID}" pid="2">{vector}</property></Properties>'),
        "variant-depth": dict(presentation=pres_doc(), custom=f'{DECL}<Properties {CP_NS}><property fmtid="{FMTID}" pid="2">' + "<vt:variant>" * 40 + "<vt:null/>" + "</vt:variant>" * 40 + "</property></Properties>"),
        "value-bytes": dict(presentation=pres_doc(), core=f"{DECL}<cp:coreProperties {CORE_NS}><dc:title>{long_title}</dc:title></cp:coreProperties>"),
    }
    for name, parts in limits.items():
        yield f"hostile/import-limit-{name}.bin", zipped(full_deck(**parts)), "open"
    # Over the bound on findings: distinct unknown elements, a report that must not grow without end.
    flood = "".join(f"<z{i}/>" for i in range(10_100))
    yield "hostile/import-findings-flood.bin", zipped(full_deck(pres_doc(f"<p:extLst/>{flood}"))), "open"
    # Values that must be reported and left out, never coerced and never a panic.
    yield "hostile/import-bad-values.bin", zipped(full_deck(**bad_values())), "open"


# ---------------------------------------------------------------- Tier 1D: slides, layouts, masters, shape trees

A16 = "http://schemas.microsoft.com/office/drawing/2014/main"
ADEC = "http://schemas.microsoft.com/office/drawing/2017/decorative"
P14 = "http://schemas.microsoft.com/office/powerpoint/2010/main"
MC_NS = "http://schemas.openxmlformats.org/markup-compatibility/2006"


def xf(x, y, cx, cy, attrs=""):
    return f'<a:xfrm{attrs}><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>'


def sp(sid, name, ph="", xfrm="", locks="", extra_nv="", cnvpr_attrs="", cnvpr_body="", attrs="", props=""):
    """A shape. `ph` is the whole `p:ph` element; `xfrm` the whole `a:xfrm`."""
    return (f'<p:sp{attrs}><p:nvSpPr><p:cNvPr id="{sid}" name="{name}"{cnvpr_attrs}>{cnvpr_body}</p:cNvPr>'
            f'<p:cNvSpPr{extra_nv}>{locks}</p:cNvSpPr><p:nvPr>{ph}</p:nvPr></p:nvSpPr>'
            f'<p:spPr{props}>{xfrm}<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
            f'<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>{name}</a:t></a:r></a:p></p:txBody></p:sp>')


def ph_tree(*shapes):
    return f'<p:spTree>{TREE}{"".join(shapes)}</p:spTree>'


LIST = '<a:lvl1pPr><a:defRPr sz="1800"/></a:lvl1pPr>'


def slides_parts():
    """Every Tier 1D construct at least once: a master with placeholders, header and footer
    flags and text styles; three layouts (one hiding master shapes, with its own background
    and colour map); three slides inheriting from them; a notes slide, notes master and
    handout master; every node kind, every lock element, hyperlinks and actions, alternate
    content, and drawing extensions. Synthetic: written to the schema, not by PowerPoint."""
    master = (
        f'{DECL}<p:sldMaster {NS} preserve="1"><p:cSld name="Fixture Master">'
        '<p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg>'
        + ph_tree(
            sp(2, "Title Placeholder 1", '<p:ph type="title"/>', xf(838200, 365125, 10515600, 1325563), '<a:spLocks noGrp="1"/>'),
            sp(3, "Text Placeholder 2", '<p:ph type="body" idx="1"/>', xf(838200, 1825625, 10515600, 4351338), '<a:spLocks noGrp="1"/>'),
            sp(4, "Date Placeholder 3", '<p:ph type="dt" sz="half" idx="2"/>', xf(838200, 6356350, 2743200, 365125)),
            sp(5, "Footer Placeholder 4", '<p:ph type="ftr" sz="quarter" idx="3"/>', xf(4038600, 6356350, 4114800, 365125)),
            sp(6, "Slide Number Placeholder 5", '<p:ph type="sldNum" sz="quarter" idx="4"/>', xf(8610600, 6356350, 2743200, 365125)),
            sp(7, "Master Logo", "", xf(11277600, 76200, 838200, 457200)))
        + f'</p:cSld>{CLRMAP}'
        '<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/><p:sldLayoutId id="2147483650" r:id="rId2"/>'
        '<p:sldLayoutId id="2147483651" r:id="rId3"/></p:sldLayoutIdLst>'
        '<p:hf hdr="0" ftr="0" dt="0"/>'
        f'<p:txStyles><p:titleStyle>{LIST}</p:titleStyle><p:bodyStyle>{LIST}</p:bodyStyle><p:otherStyle>{LIST}</p:otherStyle></p:txStyles>'
        '</p:sldMaster>')
    layout1 = (
        f'{DECL}<p:sldLayout {NS} type="title" matchingName="Title" preserve="1" userDrawn="1"><p:cSld name="Title Slide">'
        + ph_tree(
            sp(2, "Title 1", '<p:ph type="ctrTitle" orient="horz" hasCustomPrompt="1"/>', xf(1524000, 1122363, 9144000, 2387600)),
            sp(3, "Subtitle 2", '<p:ph type="subTitle" idx="1"/>', xf(1524000, 3602038, 9144000, 1655762)),
            sp(4, "Date Placeholder 3", '<p:ph type="dt" sz="half" idx="10"/>'),
            sp(5, "Footer Placeholder 4", '<p:ph type="ftr" sz="quarter" idx="11"/>'),
            sp(6, "Slide Number Placeholder 5", '<p:ph type="sldNum" sz="quarter" idx="12"/>'))
        + '</p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr><p:hf sldNum="0"/></p:sldLayout>')
    layout2 = (
        f'{DECL}<p:sldLayout {NS} type="obj" preserve="1"><p:cSld name="Title and Content">'
        + ph_tree(sp(2, "Title 1", '<p:ph type="title"/>'), sp(3, "Content Placeholder 2", '<p:ph idx="1"/>'),
                  sp(4, "Picture Placeholder 3", '<p:ph type="pic" sz="quarter" idx="13"/>', xf(838200, 4000000, 3000000, 2000000)))
        + '</p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>')
    swapped = ('<a:overrideClrMapping bg1="dk1" tx1="lt1" bg2="dk2" tx2="lt2" accent1="accent1" accent2="accent2" '
               'accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>')
    layout3 = (
        f'{DECL}<p:sldLayout {NS} type="blank" showMasterSp="0" showMasterPhAnim="0"><p:cSld name="Dark">'
        '<p:bg bwMode="black"><p:bgPr shadeToTitle="1"><a:solidFill><a:srgbClr val="1F1F1F"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>'
        + ph_tree(sp(2, "Layout Deco", "", xf(0, 0, 12192000, 228600)))
        + f'</p:cSld><p:clrMapOvr>{swapped}</p:clrMapOvr><p:hf hdr="1" ftr="1" dt="1" sldNum="1"/></p:sldLayout>')

    creation = (f'<a:extLst><a:ext uri="{{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}}"><a16:creationId xmlns:a16="{A16}" '
                'id="{6C3B0F0E-1D2A-4B5C-9D8E-0F1A2B3C4D5E}"/></a:ext>'
                f'<a:ext uri="{{C183D7F6-B498-43B3-948B-1728B52AA6E4}}"><adec:decorative xmlns:adec="{ADEC}" val="0"/></a:ext></a:extLst>')
    slide1 = (
        f'{DECL}<p:sld {NS}><p:cSld>'
        + ph_tree(
            sp(2, "Title 1", '<p:ph type="ctrTitle"/>', cnvpr_body=creation),
            sp(3, "Subtitle 2", '<p:ph type="subTitle" idx="1"/>', xf(1524000, 3900000, 9144000, 1200000),
               cnvpr_attrs=' descr="Alt text" title="Subtitle"',
               cnvpr_body='<a:hlinkClick r:id="" action="ppaction://hlinkshowjump?jump=nextslide" highlightClick="1"/>'),
            sp(4, "Footer Placeholder 3", '<p:ph type="ftr" sz="quarter" idx="11"/>'))
        + '</p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>')

    grp_inner = ('<p:grpSp><p:nvGrpSpPr><p:cNvPr id="13" name="Inner Group"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>'
                 f'<p:grpSpPr>{xf(0, 0, 914400, 914400)}</p:grpSpPr>'
                 + sp(14, "Inner Child", "", xf(0, 0, 914400, 914400)) + '</p:grpSp>')
    group = ('<p:grpSp><p:nvGrpSpPr><p:cNvPr id="10" name="Group 9"/><p:cNvGrpSpPr><a:grpSpLocks noUngrp="1" noRot="1"/></p:cNvGrpSpPr><p:nvPr/></p:nvGrpSpPr>'
             '<p:grpSpPr bwMode="auto"><a:xfrm rot="5400000" flipH="1"><a:off x="914400" y="914400"/><a:ext cx="1828800" cy="914400"/>'
             '<a:chOff x="0" y="0"/><a:chExt cx="1828800" cy="914400"/></a:xfrm></p:grpSpPr>'
             + sp(11, "Child A", "", xf(0, 0, 914400, 914400), '<a:spLocks noMove="1" noResize="1" noTextEdit="1"/>', ' txBox="1"')
             + sp(12, "Child B", "", xf(914400, 0, 914400, 914400), attrs=' useBgFill="1"', props=' bwMode="gray"')
             + grp_inner + '</p:grpSp>')
    connector = ('<p:cxnSp><p:nvCxnSpPr><p:cNvPr id="15" name="Connector 14"/><p:cNvCxnSpPr><a:cxnSpLocks noChangeShapeType="1"/>'
                 '<a:stCxn id="11" idx="3"/><a:endCxn id="12" idx="1"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr>'
                 '<p:spPr>' + xf(3000000, 3000000, 1000000, 0, ' flipV="1"') + '<a:prstGeom prst="straightConnector1"><a:avLst/></a:prstGeom></p:spPr></p:cxnSp>')
    picture = ('<p:pic><p:nvPicPr><p:cNvPr id="16" name="Picture 15" descr="A checker"/><p:cNvPicPr><a:picLocks noChangeAspect="1" noCrop="1"/></p:cNvPicPr>'
               '<p:nvPr><p:ph type="pic" sz="quarter" idx="13"/></p:nvPr></p:nvPicPr><p:blipFill><a:blip r:embed="rId3"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>'
               '<p:spPr><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')
    frame = ('<p:graphicFrame bwMode="auto"><p:nvGraphicFramePr><p:cNvPr id="17" name="Table 16"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1" noDrilldown="1"/></p:cNvGraphicFramePr>'
             '<p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="6000000" y="1825625"/><a:ext cx="4000000" cy="740000"/></p:xfrm>'
             '<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblPr firstRow="1"/>'
             '<a:tblGrid><a:gridCol w="4000000"/></a:tblGrid><a:tr h="370000"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p/></a:txBody><a:tcPr/></a:tc></a:tr>'
             '</a:tbl></a:graphicData></a:graphic></p:graphicFrame>')
    links = ('<a:hlinkClick r:id="rId4" action="ppaction://hlinksldjump" tooltip="To slide 3" history="0"/>'
             '<a:hlinkHover r:id="" action="ppaction://noaction" endSnd="1"/>')
    alternate = (f'<mc:AlternateContent xmlns:mc="{MC_NS}"><mc:Choice xmlns:p14="{P14}" Requires="p14">'
                 + sp(18, "Choice Shape", "", xf(0, 5000000, 914400, 914400)) + '</mc:Choice><mc:Fallback>'
                 + sp(18, "Fallback Shape", "", xf(0, 5000000, 914400, 914400)) + '</mc:Fallback></mc:AlternateContent>')
    slide2 = (
        f'{DECL}<p:sld {NS} xmlns:p14="{P14}" xmlns:mc="{MC_NS}" mc:Ignorable="p14"><p:cSld name="Content">'
        + ph_tree(
            sp(2, "Title 1", '<p:ph type="title"/>', cnvpr_body=links),
            sp(3, "Content Placeholder 2", '<p:ph idx="1"/>'),
            sp(4, "Orphan Placeholder", '<p:ph type="body" idx="4294967295"/>', xf(838200, 5500000, 3000000, 600000)),
            sp(5, "Hidden", "", xf(0, 0, 100, 100), cnvpr_attrs=' hidden="1"'),
            group, connector, picture, frame, alternate,
            '<p:contentPart r:id="rId5"/>')
        + '</p:cSld></p:sld>')
    slide3 = (
        f'{DECL}<p:sld {NS} show="0" showMasterSp="0" showMasterPhAnim="0"><p:cSld name="Hidden dark">'
        '<p:bg><p:bgPr><a:noFill/><a:effectLst/></p:bgPr></p:bg>'
        + ph_tree(sp(2, "Note", "", xf(914400, 914400, 4572000, 914400)))
        + f'</p:cSld><p:clrMapOvr>{swapped}</p:clrMapOvr></p:sld>')
    notes = (
        f'{DECL}<p:notes {NS}><p:cSld>'
        + ph_tree(sp(2, "Slide Image Placeholder 1", '<p:ph type="sldImg"/>', locks='<a:spLocks noGrp="1" noRot="1" noChangeAspect="1"/>'),
                  sp(3, "Notes Placeholder 2", '<p:ph type="body" idx="1"/>'))
        + '</p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:notes>')
    notes_master = (
        f'{DECL}<p:notesMaster {NS}><p:cSld>'
        + ph_tree(sp(2, "Header Placeholder 1", '<p:ph type="hdr" sz="quarter"/>', xf(0, 0, 2971800, 458788)),
                  sp(3, "Slide Image Placeholder 2", '<p:ph type="sldImg" idx="2"/>', xf(685800, 1143000, 5486400, 3086100)),
                  sp(4, "Notes Placeholder 3", '<p:ph type="body" sz="quarter" idx="3"/>', xf(685800, 4400550, 5486400, 3600450)))
        + f'</p:cSld>{CLRMAP}<p:hf dt="0"/><p:notesStyle>{LIST}</p:notesStyle></p:notesMaster>')
    handout = (
        f'{DECL}<p:handoutMaster {NS}><p:cSld>'
        + ph_tree(sp(2, "Header Placeholder 1", '<p:ph type="hdr" sz="quarter"/>', xf(0, 0, 2971800, 458788)),
                  sp(3, "Slide Number Placeholder 2", '<p:ph type="sldNum" sz="quarter" idx="3"/>', xf(3884613, 8685213, 2971800, 458787)))
        + f'</p:cSld>{CLRMAP}<p:hf hdr="0"/></p:handoutMaster>')
    presentation = (
        f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>'
        '<p:notesMasterIdLst><p:notesMasterId r:id="rId6"/></p:notesMasterIdLst><p:handoutMasterIdLst><p:handoutMasterId r:id="rId7"/></p:handoutMasterIdLst>'
        '<p:sldIdLst><p:sldId id="256" r:id="rId3"/><p:sldId id="257" r:id="rId4"/><p:sldId id="258" r:id="rId5"/></p:sldIdLst>'
        '<p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>')
    png = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                        "e221bc330000000049454e44ae426082")
    types = [(f"/ppt/slideLayouts/slideLayout{i}.xml", f"{PML}.slideLayout+xml") for i in (2, 3)]
    types += [("/ppt/notesSlides/notesSlide1.xml", f"{PML}.notesSlide+xml"),
              ("/ppt/notesMasters/notesMaster1.xml", f"{PML}.notesMaster+xml"),
              ("/ppt/handoutMasters/handoutMaster1.xml", f"{PML}.handoutMaster+xml")]
    entries = deck(3, presentation=presentation, types_extra=types)
    layout_rels = rels(("slideMaster", "../slideMasters/slideMaster1.xml"))
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),  # rId1, rId2
        ("slide", "slides/slide1.xml"), ("slide", "slides/slide2.xml"), ("slide", "slides/slide3.xml"),  # rId3-5
        ("notesMaster", "notesMasters/notesMaster1.xml"), ("handoutMaster", "handoutMasters/handoutMaster1.xml")))  # rId6, rId7
    entries = replace(entries, "ppt/slideMasters/slideMaster1.xml", master)
    entries = replace(entries, "ppt/slideMasters/_rels/slideMaster1.xml.rels", rels(
        ("slideLayout", "../slideLayouts/slideLayout1.xml"), ("slideLayout", "../slideLayouts/slideLayout2.xml"),
        ("slideLayout", "../slideLayouts/slideLayout3.xml"), ("theme", "../theme/theme1.xml")))
    entries = replace(entries, "ppt/slideLayouts/slideLayout1.xml", layout1)
    entries = replace(entries, "ppt/slides/slide1.xml", slide1)
    entries = replace(entries, "ppt/slides/slide2.xml", slide2)
    entries = replace(entries, "ppt/slides/slide3.xml", slide3)
    entries = replace(entries, "ppt/slides/_rels/slide2.xml.rels", rels(
        ("slideLayout", "../slideLayouts/slideLayout2.xml"), ("slide", "slide1.xml"), ("image", "../media/image1.png"),
        ("slide", "slide3.xml"), ("customXml", "../ink/ink1.xml")))
    entries = replace(entries, "ppt/slides/_rels/slide3.xml.rels", rels(
        ("slideLayout", "../slideLayouts/slideLayout3.xml"), ("notesSlide", "../notesSlides/notesSlide1.xml")))
    entries += [
        ("ppt/slideLayouts/slideLayout2.xml", layout2), ("ppt/slideLayouts/_rels/slideLayout2.xml.rels", layout_rels),
        ("ppt/slideLayouts/slideLayout3.xml", layout3), ("ppt/slideLayouts/_rels/slideLayout3.xml.rels", layout_rels),
        ("ppt/notesSlides/notesSlide1.xml", notes),
        ("ppt/notesSlides/_rels/notesSlide1.xml.rels", rels(("notesMaster", "../notesMasters/notesMaster1.xml"), ("slide", "../slides/slide3.xml"))),
        ("ppt/notesMasters/notesMaster1.xml", notes_master),
        ("ppt/notesMasters/_rels/notesMaster1.xml.rels", rels(("theme", "../theme/theme1.xml"))),
        ("ppt/handoutMasters/handoutMaster1.xml", handout),
        ("ppt/handoutMasters/_rels/handoutMaster1.xml.rels", rels(("theme", "../theme/theme1.xml"))),
        ("ppt/media/image1.png", png),
        ("ppt/ink/ink1.xml", '<inkml:ink xmlns:inkml="http://www.w3.org/2003/InkML"/>'),
    ]
    return entries


def slide_hostile():
    """Each opens as a package; the slide import refuses or reports. Codes are in
    crates/casual-slide-import/tests/slides.rs."""
    def with_slide(body, slide_rels=None, extra=()):
        entries = replace(deck(), "ppt/slides/slide1.xml", f"{DECL}<p:sld {NS}><p:cSld>{body}</p:cSld></p:sld>")
        if slide_rels is not None:
            entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", slide_rels)
        return zipped(list(entries) + list(extra))
    node = '<p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp>'
    yield "hostile/slide-limit-shapes.bin", with_slide(f"<p:spTree>{TREE}" + seeded(node, 100_001) + "</p:spTree>"), "open"
    depth = 70
    nested = ('<p:grpSp><p:nvGrpSpPr><p:cNvPr id="3" name="g"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>' * depth
              + "</p:grpSp>" * depth)
    yield "hostile/slide-limit-group-depth.bin", with_slide(f"<p:spTree>{TREE}{nested}</p:spTree>"), "open"
    dupes = (f"<p:spTree>{TREE}" + sp(2, "a", '<p:ph type="title"/>') + sp(2, "b", '<p:ph type="title"/>')
             + sp(3, "c", '<p:ph idx="7"/>') + sp(4, "d", '<p:ph idx="7"/>') + "</p:spTree>")
    yield "hostile/slide-duplicate-ids.bin", with_slide(dupes), "open"
    # The slide's layout relationship names the slide itself, and the layout's master
    # relationship names the slide: a cycle if a reader followed kinds blindly.
    cycle = replace(deck(), "ppt/slides/_rels/slide1.xml.rels", rels(("slideLayout", "slide1.xml")))
    cycle = replace(cycle, "ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(("slideMaster", "../slides/slide1.xml")))
    yield "hostile/slide-layout-cycle.bin", zipped(cycle), "open"
    # A layout relationship that reaches a real part of another kind, not yet seen as anything.
    yield "hostile/slide-layout-wrong-kind.bin", with_slide(f"<p:spTree>{TREE}</p:spTree>",
                                                            rels(("slideLayout", "../theme/theme1.xml"))), "open"
    # A slide with no layout relationship at all, and one whose layout part is not there.
    yield "hostile/slide-no-layout.bin", with_slide(f"<p:spTree>{TREE}</p:spTree>", rels()), "open"
    yield "hostile/slide-missing-layout.bin", with_slide(f"<p:spTree>{TREE}</p:spTree>",
                                                          rels(("slideLayout", "../slideLayouts/slideLayout9.xml"))), "open CS-FND-0004"
    bad = (f"<p:spTree>{TREE}"
           '<p:sp useBgFill="maybe"><p:nvSpPr><p:cNvPr id="x" name="n" hidden="2"><a:hlinkClick r:id="rId404" action="ppaction://program"/></p:cNvPr>'
           '<p:cNvSpPr txBox="?"><a:spLocks noMove="yes" noUngrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="nope" idx="-1" sz="huge"/></p:nvPr></p:nvSpPr>'
           '<p:spPr bwMode="sepia"><a:xfrm rot="1.5" flipH="2"><a:off x="1e99" y="0"/><a:ext cx="-5" cy="1"/></a:xfrm></p:spPr></p:sp>'
           '<p:cxnSp><p:nvCxnSpPr><p:cNvPr id="5" name="c"/><p:cNvCxnSpPr><a:stCxn id="z"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>'
           '<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="6" name="f"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
           '<p:xfrm/><a:graphic><a:graphicData/></a:graphic></p:graphicFrame><p:contentPart/><p:unknownNode/></p:spTree>')
    yield "hostile/slide-bad-values.bin", with_slide(bad), "open"


A14_NS = "http://schemas.microsoft.com/office/drawing/2010/main"
ASK_NS = "http://schemas.microsoft.com/office/drawing/2018/sketchyshapes"
PNG = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                    "e221bc330000000049454e44ae426082")


def drawn(sid, name, props, tag="sp", holder="spPr", nv="cNvSpPr", container="nvSpPr"):
    return (f'<p:{tag}><p:{container}><p:cNvPr id="{sid}" name="{name}"/><p:{nv}/><p:nvPr/></p:{container}>'
            f'<p:{holder}><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm>{props}</p:{holder}></p:{tag}>')


RECT = '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom>'
RGB = '<a:srgbClr val="112233"><a:alpha val="50000"/></a:srgbClr>'
SOLID = f'<a:solidFill>{RGB}</a:solidFill>'
LIST_EFFECTS = (
    '<a:blur rad="1000" grow="0"/>'
    f'<a:fillOverlay blend="mult">{SOLID}</a:fillOverlay>'
    f'<a:glow rad="2000">{RGB}</a:glow>'
    f'<a:innerShdw blurRad="1" dist="2" dir="3">{RGB}</a:innerShdw>'
    f'<a:outerShdw blurRad="1" dist="2" dir="3" sx="90000" sy="80000" kx="4" ky="5" algn="ctr" rotWithShape="0">{RGB}</a:outerShdw>'
    f'<a:prstShdw prst="shdw7" dist="2" dir="3">{RGB}</a:prstShdw>'
    '<a:reflection blurRad="1" stA="2" stPos="3" endA="4" endPos="5" dist="6" dir="7" fadeDir="8" sx="9" sy="10" kx="11" ky="12" algn="tl" rotWithShape="1"/>'
    '<a:softEdge rad="1234"/>')
BLIP_EFFECTS = (
    '<a:alphaBiLevel thresh="50000"/><a:alphaCeiling/><a:alphaFloor/>'
    f'<a:alphaInv>{RGB}</a:alphaInv>'
    '<a:alphaMod><a:cont type="sib" name="m"><a:alphaFloor/></a:cont></a:alphaMod>'
    '<a:alphaModFix amt="90000"/><a:alphaRepl a="20000"/><a:biLevel thresh="30000"/><a:blur rad="9"/>'
    f'<a:clrChange useA="0"><a:clrFrom>{RGB}</a:clrFrom><a:clrTo>{RGB}</a:clrTo></a:clrChange>'
    f'<a:clrRepl>{RGB}</a:clrRepl><a:duotone>{RGB}<a:schemeClr val="accent1"/></a:duotone>'
    f'<a:fillOverlay blend="screen">{SOLID}</a:fillOverlay><a:grayscl/>'
    '<a:hsl hue="60000" sat="10000" lum="-10000"/><a:lum bright="10000" contrast="-20000"/><a:tint hue="1" amt="2"/>')
ARTISTIC = "".join(
    f'<a14:imgEffect visible="1"><a14:{name}{attrs}/></a14:imgEffect>' for name, attrs in [
        ("artisticBlur", ' radius="10"'), ("artisticCement", ' trans="1000" crackSpacing="24"'),
        ("artisticChalkSketch", ' trans="1000" pressure="2"'), ("artisticCrisscrossEtching", ' trans="1000" pressure="30"'),
        ("artisticCutout", ' trans="1000" numberOfShades="2"'), ("artisticFilmGrain", ' trans="1000" grainSize="40"'),
        ("artisticGlass", ' trans="1000" scaling="34"'), ("artisticGlowDiffused", ' trans="1000" intensity="5"'),
        ("artisticGlowEdges", ' trans="1000" smoothness="3"'), ("artisticLightScreen", ' trans="1000" gridSize="4"'),
        ("artisticLineDrawing", ' trans="1000" pencilSize="9"'), ("artisticMarker", ' trans="1000" size="97"'),
        ("artisticMosiaicBubbles", ' trans="1000" pressure="14"'), ("artisticPaintStrokes", ' trans="1000" intensity="5"'),
        ("artisticPaintBrush", ' trans="1000" brushSize="2"'), ("artisticPastelsSmooth", ' trans="1000" scaling="34"'),
        ("artisticPencilGrayscale", ' trans="1000" pencilSize="27"'), ("artisticPencilSketch", ' trans="1000" pressure="22"'),
        ("artisticPhotocopy", ' trans="1000" detail="3"'), ("artisticPlasticWrap", ' trans="1000" smoothness="5"'),
        ("artisticTexturizer", ' trans="1000" scaling="34"'), ("artisticWatercolorSponge", ' trans="1000" brushSize="2"'),
        ("brightnessContrast", ' bright="10000" contrast="-10000"'), ("colorTemperature", ' colorTemp="6500"'),
        ("saturation", ' sat="150000"'), ("sharpenSoften", ' amount="25000"')])
IMG_PROPS = (f'<a14:imgProps xmlns:a14="{A14_NS}"><a14:imgLayer r:embed="rId2">{ARTISTIC}'
             '<a14:imgEffect><a14:backgroundRemoval t="1000" b="2000" l="3000" r="4000">'
             '<a14:foregroundMark x1="1" y1="2" x2="3" y2="4"/><a14:backgroundMark x1="5" y1="6" x2="7" y2="8"/>'
             '</a14:backgroundRemoval></a14:imgEffect></a14:imgLayer></a14:imgProps>')
EXT_BLIP = (f'<a:extLst><a:ext uri="{{BEBA8EAE-BF5A-486C-A8C5-ECC9F3942E4B}}">{IMG_PROPS}</a:ext>'
            f'<a:ext uri="{{28A0092B-C50C-407E-A947-70E740481C1C}}"><a14:useLocalDpi xmlns:a14="{A14_NS}" val="0"/></a:ext></a:extLst>')
SCENE = ('<a:camera prst="perspectiveFront" fov="5400000" zoom="110000"><a:rot lat="1" lon="2" rev="3"/></a:camera>'
         '<a:lightRig rig="threePt" dir="t"><a:rot lat="4" lon="5" rev="6"/></a:lightRig>'
         '<a:backdrop><a:anchor x="1" y="2" z="3"/><a:norm dx="4" dy="5" dz="6"/><a:up dx="7" dy="8" dz="9"/></a:backdrop>')
SP3D_ATTRS = ' z="10" extrusionH="20" contourW="30" prstMaterial="metal"'
SP3D_KIDS = (f'<a:bevelT w="1" h="2" prst="angle"/><a:bevelB w="3" h="4" prst="convex"/>'
             f'<a:extrusionClr>{RGB}</a:extrusionClr><a:contourClr>{RGB}</a:contourClr>')
SKETCH = (f'<a:ext uri="{{C807C97D-BFC1-408E-A445-0C87EB9F89A2}}"><ask:lineSketchStyleProps xmlns:ask="{ASK_NS}" sd="12345">'
          f'{RECT}<ask:type><ask:lineSketchScribble/></ask:type></ask:lineSketchStyleProps></a:ext>')


def drawing_slide():
    """Every Tier 1E element and attribute at least once, to the schema. Synthetic: not
    written by PowerPoint, which is what fixtures/real is for."""
    custom = ('<a:custGeom><a:avLst><a:gd name="adj" fmla="val 5000"/></a:avLst><a:gdLst><a:gd name="g1" fmla="*/ w adj 100000"/></a:gdLst>'
              '<a:ahLst><a:ahXY gdRefX="adj" minX="0" maxX="100000" gdRefY="adj" minY="0" maxY="100000"><a:pos x="g1" y="t"/></a:ahXY>'
              '<a:ahPolar gdRefR="adj" minR="0" maxR="50000" gdRefAng="adj" minAng="0" maxAng="21599999"><a:pos x="g1" y="b"/></a:ahPolar></a:ahLst>'
              '<a:cxnLst><a:cxn ang="0"><a:pos x="r" y="vc"/></a:cxn></a:cxnLst><a:rect l="l" t="t" r="r" b="b"/>'
              '<a:pathLst><a:path w="100" h="100" fill="darken" stroke="0" extrusionOk="0"><a:moveTo><a:pt x="0" y="50"/></a:moveTo>'
              '<a:lnTo><a:pt x="25" y="0"/></a:lnTo><a:arcTo wR="10" hR="20" stAng="0" swAng="5400000"/>'
              '<a:quadBezTo><a:pt x="1" y="2"/><a:pt x="3" y="4"/></a:quadBezTo>'
              '<a:cubicBezTo><a:pt x="5" y="6"/><a:pt x="7" y="8"/><a:pt x="9" y="10"/></a:cubicBezTo><a:close/></a:path></a:pathLst></a:custGeom>')
    dag = ('<a:effectDag type="tree" name="top">'
           '<a:cont type="sib" name="inner"><a:alphaFloor/></a:cont><a:effect ref="inner"/>'
           '<a:alphaBiLevel thresh="1"/><a:alphaCeiling/><a:alphaFloor/>'
           f'<a:alphaInv>{RGB}</a:alphaInv><a:alphaMod><a:cont/></a:alphaMod><a:alphaModFix amt="2"/><a:alphaOutset rad="3"/>'
           '<a:alphaRepl a="4"/><a:biLevel thresh="5"/><a:blend blend="darken"><a:cont/></a:blend><a:blur rad="6" grow="1"/>'
           f'<a:clrChange useA="1"><a:clrFrom>{RGB}</a:clrFrom><a:clrTo>{RGB}</a:clrTo></a:clrChange><a:clrRepl>{RGB}</a:clrRepl>'
           f'<a:duotone>{RGB}{RGB}</a:duotone><a:fill>{SOLID}</a:fill><a:fillOverlay blend="lighten">{SOLID}</a:fillOverlay>'
           f'<a:glow rad="7">{RGB}</a:glow><a:grayscl/><a:hsl hue="8" sat="9" lum="10"/><a:innerShdw blurRad="11">{RGB}</a:innerShdw>'
           f'<a:lum bright="12" contrast="13"/><a:outerShdw blurRad="14">{RGB}</a:outerShdw><a:prstShdw prst="shdw1">{RGB}</a:prstShdw>'
           '<a:reflection blurRad="15"/><a:relOff tx="16" ty="17"/><a:softEdge rad="18"/><a:tint hue="19" amt="20"/>'
           '<a:xfrm sx="21" sy="22" kx="23" ky="24" tx="25" ty="26"/></a:effectDag>')
    hidden = (f'<a:extLst><a:ext uri="{{909E8E84-426E-40DD-AFC4-6F175D3DCCD1}}"><a14:hiddenFill xmlns:a14="{A14_NS}">{SOLID}</a14:hiddenFill></a:ext>'
              f'<a:ext uri="{{91240B29-F687-4F45-9708-019B960494DF}}"><a14:hiddenLine xmlns:a14="{A14_NS}" w="9525">{SOLID}<a:miter lim="800000"/></a14:hiddenLine></a:ext>'
              f'<a:ext uri="{{AF507438-7753-43E0-B8FC-AC1667EBCBE1}}"><a14:hiddenEffects xmlns:a14="{A14_NS}"><a:effectLst><a:blur rad="1"/></a:effectLst></a14:hiddenEffects></a:ext>'
              f'<a:ext uri="{{53640926-AAD7-44D8-BBD7-CCE9431645EC}}"><a14:shadowObscured xmlns:a14="{A14_NS}" val="1"/></a:ext>'
              f'<a:ext uri="{{00000000-0000-0000-0000-000000000001}}"><a14:hiddenScene3d xmlns:a14="{A14_NS}">{SCENE}</a14:hiddenScene3d></a:ext>'
              f'<a:ext uri="{{00000000-0000-0000-0000-000000000002}}"><a14:hiddenSp3d xmlns:a14="{A14_NS}"{SP3D_ATTRS}>{SP3D_KIDS}</a14:hiddenSp3d></a:ext></a:extLst>')
    shapes = [
        drawn(2, "custom", custom),
        drawn(3, "solid and full line",
              '<a:prstGeom prst="wedgeRectCallout"><a:avLst><a:gd name="adj1" fmla="val -20833"/><a:gd name="adj2" fmla="val 62500"/></a:avLst></a:prstGeom>'
              f'{SOLID}<a:ln w="12700" cap="rnd" cmpd="dbl" algn="in">{SOLID}<a:prstDash val="sysDot"/><a:round/>'
              f'<a:headEnd type="triangle" w="sm" len="lg"/><a:tailEnd type="arrow" w="lg" len="sm"/><a:extLst>{SKETCH}</a:extLst></a:ln>'),
        drawn(4, "linear gradient",
              f'{RECT}<a:gradFill flip="xy" rotWithShape="1"><a:gsLst><a:gs pos="0">{RGB}</a:gs><a:gs pos="100000"><a:schemeClr val="accent1"/></a:gs></a:gsLst>'
              '<a:lin ang="5400000" scaled="0"/><a:tileRect l="1" t="2" r="3" b="4"/></a:gradFill>'
              '<a:ln><a:noFill/><a:custDash><a:ds d="100000" sp="50000"/><a:ds d="200000" sp="60000"/></a:custDash><a:miter lim="800000"/></a:ln>'),
        drawn(5, "path gradient",
              f'{RECT}<a:gradFill><a:gsLst><a:gs pos="0">{RGB}</a:gs><a:gs pos="100000">{RGB}</a:gs></a:gsLst>'
              '<a:path path="circle"><a:fillToRect l="50000" t="50000" r="50000" b="50000"/></a:path></a:gradFill>'
              f'<a:ln><a:gradFill><a:gsLst><a:gs pos="0">{RGB}</a:gs><a:gs pos="100000">{RGB}</a:gs></a:gsLst></a:gradFill><a:bevel/></a:ln>'),
        drawn(6, "pattern", f'{RECT}<a:pattFill prst="dkDnDiag"><a:fgClr>{RGB}</a:fgClr><a:bgClr><a:schemeClr val="bg1"/></a:bgClr></a:pattFill>'
              f'<a:ln><a:pattFill prst="pct5"><a:fgClr>{RGB}</a:fgClr><a:bgClr>{RGB}</a:bgClr></a:pattFill></a:ln>'),
        drawn(7, "tiled picture", f'{RECT}<a:blipFill dpi="96" rotWithShape="0"><a:blip r:embed="rId2" r:link="rId3" cstate="print"/>'
              '<a:srcRect l="1" t="2" r="3" b="4"/><a:tile tx="1" ty="2" sx="50000" sy="60000" flip="xy" algn="ctr"/></a:blipFill>'),
        drawn(8, "stretched picture", f'{RECT}<a:blipFill><a:blip r:embed="rId2">{BLIP_EFFECTS}{EXT_BLIP}</a:blip>'
              '<a:stretch><a:fillRect l="1" t="2" r="3" b="4"/></a:stretch></a:blipFill>'),
        drawn(9, "effect list and 3D", f'{RECT}<a:grpFill/><a:effectLst>{LIST_EFFECTS}</a:effectLst><a:scene3d>{SCENE}</a:scene3d><a:sp3d{SP3D_ATTRS}>{SP3D_KIDS}</a:sp3d>'),
        drawn(10, "effect dag", f'{RECT}{dag}'),
        drawn(11, "hidden", f'{RECT}<a:noFill/>{hidden}'),
        drawn(12, "connector", f'<a:prstGeom prst="bentConnector3"><a:avLst/></a:prstGeom><a:ln w="1"/>', tag="cxnSp", nv="cNvCxnSpPr", container="nvCxnSpPr"),
        *[drawn(20 + i, f"sketch {kind}", f'{RECT}<a:ln><a:extLst><a:ext uri="{{C807C97D-BFC1-408E-A445-0C87EB9F89A2}}"><ask:lineSketchStyleProps xmlns:ask="{ASK_NS}">'
                f'<ask:type><ask:{kind}/></ask:type><ask:seed>{i}</ask:seed></ask:lineSketchStyleProps></a:ext></a:extLst></a:ln>')
          for i, kind in enumerate(["lineSketchNone", "lineSketchCurved", "lineSketchFreehand"])],
        (f'<p:pic><p:nvPicPr><p:cNvPr id="13" name="picture"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill rotWithShape="1"><a:blip r:embed="rId2"><a:grayscl/>{EXT_BLIP}</a:blip>'
         f'<a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr>{RECT}</p:spPr></p:pic>'),
        ('<p:grpSp><p:nvGrpSpPr><p:cNvPr id="14" name="group"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>'
         f'<p:grpSpPr>{SOLID}<a:effectLst><a:outerShdw blurRad="1" sx="90000" sy="90000">{RGB}</a:outerShdw></a:effectLst><a:scene3d>{SCENE}</a:scene3d></p:grpSpPr>'
         + drawn(15, "in group", f'{RECT}<a:grpFill/>') + '</p:grpSp>'),
    ]
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


def drawing_deck(slide=None):
    entries = deck(slide_rels=[("image", "../media/image1.png"), ("image", "https://example.invalid/a.png", "External")],
                   parts=[("ppt/media/image1.png", PNG)])
    return replace(entries, "ppt/slides/slide1.xml", slide or drawing_slide())


def drawing_hostile():
    """Each opens; the slide part reads, with the property left out or the value reported.
    Codes are in crates/casual-slide-import/tests/drawing.rs."""
    def one(props):
        return zipped(drawing_deck(f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{drawn(2, "s", props)}</p:spTree></p:cSld></p:sld>'))
    stop = f'<a:gs pos="0">{RGB}</a:gs>'
    # Each of these carries the fixture's external image relationship, reported on opening.
    yield "hostile/drawing-limit-gradient-stops.bin", one(f'<a:gradFill><a:gsLst>{stop * 1025}</a:gsLst></a:gradFill>'), "open CS-FND-0001"
    yield "hostile/drawing-limit-dash-stops.bin", one('<a:ln><a:custDash>' + '<a:ds d="1" sp="1"/>' * 1025 + '</a:custDash></a:ln>'), "open CS-FND-0001"
    yield "hostile/drawing-limit-effects.bin", one('<a:effectDag>' + '<a:grayscl/>' * 1025 + '</a:effectDag>'), "open CS-FND-0001"
    yield "hostile/drawing-limit-effect-depth.bin", one('<a:effectDag>' + '<a:cont>' * 40 + '</a:cont>' * 40 + '</a:effectDag>'), "open CS-FND-0001"
    nested = '<a:fillOverlay blend="over"><a:blipFill><a:blip>' * 40 + '</a:blip></a:blipFill></a:fillOverlay>' * 40
    yield "hostile/drawing-limit-fill-depth.bin", one(f'<a:blipFill><a:blip>{nested}</a:blip></a:blipFill>'), "open CS-FND-0001"
    guide = '<a:gd name="a" fmla="val 1"/>'
    yield "hostile/drawing-limit-guides.bin", one(f'<a:custGeom><a:gdLst>{guide * 4097}</a:gdLst><a:pathLst/></a:custGeom>'), "open CS-FND-0001"
    path = '<a:path><a:close/></a:path>'
    yield "hostile/drawing-limit-paths.bin", one(f'<a:custGeom><a:pathLst>{path * 1025}</a:pathLst></a:custGeom>'), "open CS-FND-0001"
    cmd = '<a:lnTo><a:pt x="1" y="1"/></a:lnTo>'
    yield "hostile/drawing-limit-path-commands.bin", one(f'<a:custGeom><a:pathLst><a:path>{cmd * 262145}</a:path></a:pathLst></a:custGeom>'), "open CS-FND-0001"
    handle = '<a:ahXY><a:pos x="1" y="1"/></a:ahXY>'
    yield "hostile/drawing-limit-handles.bin", one(f'<a:custGeom><a:ahLst>{handle * 257}</a:ahLst><a:pathLst/></a:custGeom>'), "open CS-FND-0001"
    site = '<a:cxn ang="0"><a:pos x="1" y="1"/></a:cxn>'
    yield "hostile/drawing-limit-sites.bin", one(f'<a:custGeom><a:cxnLst>{site * 4097}</a:cxnLst><a:pathLst/></a:custGeom>'), "open CS-FND-0001"
    bad = ('<a:prstGeom prst="notAShape"><a:avLst><a:gd/><a:gd name="zz" fmla="val 1"/><a:gd name="yy" fmla="bogus 1 2"/></a:avLst></a:prstGeom>'
           '<a:gradFill flip="q" rotWithShape="9"><a:gsLst><a:gs pos="999999"/><a:gs/><a:gs pos="x"><a:srgbClr val="nope"/></a:gs></a:gsLst>'
           '<a:lin ang="-1" scaled="?"/><a:path path="wat"/></a:gradFill><a:solidFill><a:srgbClr val="000000"/></a:solidFill>'
           '<a:ln w="99999999" cap="z" cmpd="z" algn="z"><a:blipFill/><a:prstDash val="z"/><a:custDash><a:ds d="0" sp="-1"/></a:custDash>'
           '<a:round/><a:miter lim="-5"/><a:headEnd type="z" w="z" len="z"/><a:extLst><a:ext uri="x"><a:foreign/></a:ext></a:extLst></a:ln>'
           '<a:effectLst><a:outerShdw dir="99999999999" kx="5400000" blurRad="-1" sx="1e9"/><a:outerShdw/><a:tint/><a:softEdge/>'
           '<a:fillOverlay/></a:effectLst><a:effectDag name="a"><a:cont name="a"><a:effect ref="nowhere"/></a:cont></a:effectDag>'
           '<a:scene3d><a:camera prst="z" fov="-1" zoom="-1"><a:rot lat="-1"/></a:camera><a:lightRig rig="z" dir="z"/></a:scene3d>'
           '<a:sp3d z="1e99" extrusionH="-1" contourW="-1" prstMaterial="z"><a:bevelT w="-1" prst="z"/><a:extrusionClr/></a:sp3d>'
           '<a:unknownThing/><a:extLst><a:ext uri="y"><foreign xmlns="urn:x"/></a:ext></a:extLst>')
    yield "hostile/drawing-bad-values.bin", one(bad), "open CS-FND-0001"
    cg = ('<a:custGeom><a:gdLst><a:gd name="a" fmla="wat a b"/><a:gd name="c" fmla="*/ a nothere"/></a:gdLst><a:rect/><a:ahLst><a:ahXY/><a:wat/></a:ahLst>'
          '<a:cxnLst><a:cxn/></a:cxnLst><a:pathLst><a:path w="-1" fill="z" stroke="9"><a:lnTo/><a:quadBezTo/><a:cubicBezTo><a:pt/></a:cubicBezTo><a:arcTo/><a:wat/></a:path></a:pathLst></a:custGeom>')
    yield "hostile/drawing-bad-custom-geometry.bin", one(cg), "open CS-FND-0001"
    # A blip naming no relationship, and a DAG whose containers reference each other.
    yield "hostile/drawing-dangling-and-cycle.bin", one(
        '<a:blipFill><a:blip r:embed="rId404" r:link="rId405"/></a:blipFill>'
        '<a:effectDag name="a"><a:effect ref="a"/><a:cont name="b"><a:effect ref="a"/><a:effect ref="b"/></a:cont></a:effectDag>'), "open CS-FND-0001"


def hostile_noise():
    # A single value of 16 MiB compresses past the ratio bound unless part of it does not.
    return "".join(hashlib.sha256(str(i).encode()).hexdigest() for i in range(1024))


def bad_values():
    presentation = (
        f'{DECL}<p:presentation {NS} serverZoom="lots" firstSlideNum="99999999999999999999" rtl="maybe" bookmarkIdSeed="0" conformance="loose" compatMode="">'
        '<p:sldMasterIdLst><p:sldMasterId id="5" r:id="rId1"/><p:sldMasterId id="NaN" r:id="rId1"/></p:sldMasterIdLst>'
        '<p:sldIdLst><p:sldId id="4294967295" r:id="rId3"/><p:sldId id="5" r:id="rId4"/><p:sldId id="256" r:id="rId999"/><p:sldId id="256" r:id="rId1"/>'
        '<p:sldId id="-1" r:id="rId5"/><p:sldId id="4294967296" r:id="rId5"/><p:sldId r:id="rId5"/><p:sldId id="300"/></p:sldIdLst>'
        '<p:sldSz cx="1" cy="99999999999" type="poster"/><p:notesSz cx="-1" cy="1e9"/>'
        '<p:embeddedFontLst><p:embeddedFont/><p:embeddedFont><p:font typeface="F" panose="zz" pitchFamily="999" charset="1000"/><p:regular r:id="rId2"/></p:embeddedFont></p:embeddedFontLst>'
        '<p:custShowLst><p:custShow name="s"/><p:custShow id="1" name="a"><p:sldLst><p:sld r:id="rId2"/></p:sldLst></p:custShow><p:custShow id="1" name="b"/></p:custShowLst>'
        '<p:photoAlbum layout="9pic" frame=""/><p:kinsoku lang="x"/><p:modifyVerifier spinCount="-3" cryptProviderType="rot13"/>'
        '<p:sldSz cx="914400" cy="914400"/></p:presentation>')
    pres_props = (f'{DECL}<p:presentationPr {NS}><p:webPr imgSz="1x1" clr="puce"/><p:prnPr prnWhat="all" clrMode="sepia"/>'
                  '<p:showPr loop="2"><p:kiosk restart="-1"/><p:browse/><p:sldRg st="9" end="1"/><p:custShow id="777"/></p:showPr>'
                  '<p:htmlPubPr r:id="rId404"><p:sldRg st="1" end="4000000000"/></p:htmlPubPr></p:presentationPr>')
    view_props = (f'{DECL}<p:viewPr {NS} lastView="nope" showComments="2"><p:normalViewPr vertBarState="z"><p:restoredLeft sz="200000"/><p:restoredTop/></p:normalViewPr>'
                  '<p:slideViewPr><p:cSldViewPr><p:cViewPr><p:scale><a:sx n="x" d="1"/></p:scale><p:origin x="1e400in" y="99999999999999999999"/></p:cViewPr>'
                  '<p:guideLst><p:guide pos="99999999999"/><p:guide orient="diag" pos="1zz"/></p:guideLst></p:cSldViewPr></p:slideViewPr>'
                  '<p:outlineViewPr><p:cViewPr/><p:sldLst><p:sld r:id="rId404"/><p:sld/></p:sldLst></p:outlineViewPr><p:gridSpacing cx="-5" cy="5"/></p:viewPr>')
    core = (f'{DECL}<cp:coreProperties {CORE_NS}><dc:title><b>no</b></dc:title><dc:title>second</dc:title><dcterms:created>yesterday</dcterms:created>'
            '<dcterms:modified xsi:type="dcterms:W3CDTF">2021-02-30T25:00:00Z</dcterms:modified><cp:lastPrinted>0</cp:lastPrinted>'
            '<title>no namespace</title></cp:coreProperties>')
    app = (f'{DECL}<Properties {EP_NS}><Slides>many</Slides><Notes>99999999999</Notes><ScaleCrop>2</ScaleCrop>'
           '<TitlesOfParts><vt:vector size="4294967295" baseType="lpstr"><vt:lpstr>a</vt:lpstr></vt:vector></TitlesOfParts>'
           '<HeadingPairs><vt:vector size="zero" baseType="wat"><vt:i4>x</vt:i4></vt:vector></HeadingPairs><HLinks/><DigSig/></Properties>')
    custom = (f'{DECL}<Properties {CP_NS}><property/><property fmtid="nope" pid="2"/><property fmtid="{FMTID}" pid="x"/>'
              f'<property fmtid="{FMTID}" pid="1"/><property fmtid="{FMTID}" pid="3"><vt:r8>inf</vt:r8></property>'
              f'<property fmtid="{FMTID}" pid="3"><vt:i1>128</vt:i1></property><property fmtid="{FMTID}" pid="4"><vt:cy>1.5</vt:cy></property>'
              f'<property fmtid="{FMTID}" pid="5"><vt:date>not a date</vt:date></property><property fmtid="{FMTID}" pid="6"><vt:blob>!!!</vt:blob></property>'
              f'<property fmtid="{FMTID}" pid="7"><vt:array lBounds="a" uBounds="1" baseType="i4"/></property><property fmtid="{FMTID}" pid="8"><vt:wat/></property></Properties>')
    return dict(presentation=presentation, core=core, app=app, custom=custom, pres_props=pres_props, view_props=view_props)


def check():
    """Committed bytes against the manifest. Deliberately not a regeneration, since
    deflate output can differ between zlib versions."""
    rows = [ln.split("\t") for ln in (ROOT / "manifest.tsv").read_text().splitlines()[1:]]
    listed = {r[0] for r in rows}
    bad = [r[0] for r in rows if not (ROOT / r[0]).exists() or hashlib.sha256((ROOT / r[0]).read_bytes()).hexdigest() != r[1]]
    stray = [str(p.relative_to(ROOT)) for d in ("generated", "hostile", "producers", "real") for p in (ROOT / d).glob("*") if str(p.relative_to(ROOT)) not in listed and p.name not in ("expect.tsv", "README.md")]
    if bad or stray:
        raise SystemExit(f"fixtures differ from manifest: {bad} unlisted: {stray}")
    print(f"fixtures match manifest: {len(rows)}")


def main():
    if "--check" in sys.argv:
        check()
        return
    lines = ["path\tsha256\texpect\n"]
    for rel, data, expect, *limits in fixtures():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        extra = "".join(f"\t{x}" for x in limits)
        lines.append(f"{rel}\t{hashlib.sha256(data).hexdigest()}\t{expect}{extra}\n")
    # Decks written by other software are committed, never regenerated.
    for folder in ("producers", "real"):
        for row in (ROOT / folder / "expect.tsv").read_text().splitlines():
            name, expect = row.split("\t")
            rel = f"{folder}/{name}"
            lines.append(f"{rel}\t{hashlib.sha256((ROOT / rel).read_bytes()).hexdigest()}\t{expect}\n")
    (ROOT / "manifest.tsv").write_text("".join(lines))
    print(f"wrote {len(lines) - 1} fixtures")


if __name__ == "__main__":
    main()
