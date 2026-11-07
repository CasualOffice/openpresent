"""The input decks of the repair-risk fixtures (`RR-*`, docs/36).

Each is a deck the writer edits, in `crates/casual-slide-export/tests/writer_rr.rs`, which
writes the good and seeded pairs to fixtures/rr. Python builds what goes in; only the
writer can build what comes out.
"""
from generate import (CLRMAP, DECL, NS, PKG_RELS, PML, T, THEME, TITLE, TREE, deck, rels, replace, sp, zipped)

P14 = "http://schemas.microsoft.com/office/powerpoint/2010/main"
THEME_TYPE = "application/vnd.openxmlformats-officedocument.theme+xml"
PNG = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                    "e221bc330000000049454e44ae426082")
SECTION_A = "{11111111-AAAA-4BBB-8CCC-000000000001}"
SECTION_B = "{22222222-AAAA-4BBB-8CCC-000000000002}"


def pres(body_before="", after_notes="", slides=(), extra_rels=0, masters='<p:sldMasterId id="2147483648" r:id="rId1"/>', attrs=""):
    """A presentation part, with `slides` as (id, rId)."""
    ids = "".join(f'<p:sldId id="{i}" r:id="{r}"/>' for i, r in slides)
    return (f'{DECL}<p:presentation {NS}{attrs}><p:sldMasterIdLst>{masters}</p:sldMasterIdLst>{body_before}'
            f'<p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>'
            f'{after_notes}</p:presentation>')


def sections(*rows):
    """The section list, as (name, id, [slide ids])."""
    out = []
    for name, sid, slides in rows:
        ids = "".join(f'<p14:sldId id="{s}"/>' for s in slides)
        out.append(f'<p14:section name="{name}" id="{sid}"><p14:sldIdLst>{ids}</p14:sldIdLst></p14:section>')
    return ('<p:extLst><p:ext uri="{521415D9-36F7-43E2-AB2F-B90AF26B5E84}">'
            f'<p14:sectionLst xmlns:p14="{P14}">{"".join(out)}</p14:sectionLst></p:ext></p:extLst>')


def with_override(entries, *pairs):
    """Add `Override`s to the content types stream."""
    extra = "".join(f'<Override PartName="{n}" ContentType="{t}"/>' for n, t in pairs)
    out = []
    for name, data in entries:
        if name == "[Content_Types].xml":
            data = data.replace("</Types>", f"{extra}</Types>")
        out.append((name, data))
    return out


def with_default(entries, ext, ct):
    return [(n, d.replace("<Override", f'<Default Extension="{ext}" ContentType="{ct}"/><Override', 1) if n == "[Content_Types].xml" else d)
            for n, d in entries]


def slide_with(body, title="Slide"):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % title}{body}</p:spTree></p:cSld></p:sld>'


def picture(sid, rid):
    return (f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="Picture {sid}"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
            f'<p:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>'
            '<p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm>'
            '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')


def sectioned():
    entries = deck(3)
    pres_rels = rels(("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),
                     ("slide", "slides/slide1.xml"), ("slide", "slides/slide2.xml"), ("slide", "slides/slide3.xml"),
                     ("presProps", "presProps.xml"))
    shows = '<p:custShowLst><p:custShow name="Show" id="0"><p:sldLst><p:sld r:id="rId4"/></p:sldLst></p:custShow></p:custShowLst>'
    main = pres(slides=[(256, "rId3"), (257, "rId4"), (258, "rId5")],
                after_notes=shows + sections(("First", SECTION_A, [256, 257]), ("Second", SECTION_B, [258])))
    props = f'{DECL}<p:presentationPr {NS}><p:showPr><p:custShow id="0"/></p:showPr></p:presentationPr>'
    entries = replace(entries, "ppt/presentation.xml", main)
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", pres_rels)
    entries.append(("ppt/presProps.xml", props))
    return with_override(entries, ("/ppt/presProps.xml", f"{PML}.presProps+xml"))


def one_section():
    entries = deck(2)
    main = pres(slides=[(256, "rId3"), (257, "rId4")], after_notes=sections(("Only", SECTION_A, [256, 257])))
    return replace(entries, "ppt/presentation.xml", main)


NOTES_MASTER = (f'{DECL}<p:notesMaster {NS}><p:cSld><p:spTree>{TREE}'
                + "".join(sp(i + 2, f"Placeholder {i}", ph=f'<p:ph type="{kind}"/>') for i, kind in
                          enumerate(("hdr", "dt", "sldImg", "body", "ftr", "sldNum")))
                + f'</p:spTree></p:cSld>{CLRMAP}</p:notesMaster>')
NOTES = (f'{DECL}<p:notes {NS}><p:cSld><p:spTree>{TREE}'
         + sp(2, "Slide Image", ph='<p:ph type="sldImg"/>') + sp(3, "Notes", ph='<p:ph type="body" idx="1"/>')
         + '</p:spTree></p:cSld></p:notes>')
AUTHORS = f'{DECL}<p:cmAuthorLst {NS}><p:cmAuthor id="0" name="A" initials="A" lastIdx="1" clrIdx="0"/></p:cmAuthorLst>'
COMMENTS = (f'{DECL}<p:cmLst {NS}><p:cm authorId="0" dt="2020-01-01T00:00:00.000" idx="1">'
            '<p:pos x="10" y="10"/><p:text>Note</p:text></p:cm></p:cmLst>')


def noted():
    entries = deck(2, slide_rels=())
    pres_rels = rels(("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),
                     ("slide", "slides/slide1.xml"), ("slide", "slides/slide2.xml"),
                     ("notesMaster", "notesMasters/notesMaster1.xml"), ("commentAuthors", "commentAuthors.xml"))
    main = pres(body_before='<p:notesMasterIdLst><p:notesMasterId r:id="rId5"/></p:notesMasterIdLst>',
                slides=[(256, "rId3"), (257, "rId4")])
    entries = replace(entries, "ppt/presentation.xml", main)
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", pres_rels)
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels",
                      rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("notesSlide", "../notesSlides/notesSlide1.xml"),
                           ("comments", "../comments/comment1.xml")))
    entries += [("ppt/notesMasters/notesMaster1.xml", NOTES_MASTER),
                ("ppt/notesMasters/_rels/notesMaster1.xml.rels", rels(("theme", "../theme/theme2.xml"))),
                ("ppt/theme/theme2.xml", THEME),
                ("ppt/notesSlides/notesSlide1.xml", NOTES),
                ("ppt/notesSlides/_rels/notesSlide1.xml.rels",
                 rels(("notesMaster", "../notesMasters/notesMaster1.xml"), ("slide", "../slides/slide1.xml"))),
                ("ppt/commentAuthors.xml", AUTHORS),
                ("ppt/comments/comment1.xml", COMMENTS)]
    return with_override(entries,
                         ("/ppt/notesMasters/notesMaster1.xml", f"{PML}.notesMaster+xml"),
                         ("/ppt/theme/theme2.xml", THEME_TYPE),
                         ("/ppt/notesSlides/notesSlide1.xml", f"{PML}.notesSlide+xml"),
                         ("/ppt/commentAuthors.xml", f"{PML}.commentAuthors+xml"),
                         ("/ppt/comments/comment1.xml", f"{PML}.comments+xml"))


def shared_media():
    """Two slides on one image, and the theme on the same image and on one of its own."""
    entries = deck(2, slide_rels=[("image", "../media/image1.png")])
    entries = replace(entries, "ppt/slides/slide1.xml", slide_with(picture(4, "rId2") + picture(5, "rId3"), "One"))
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels",
                      rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("image", "../media/image1.png"), ("image", "../media/image2.png")))
    entries = replace(entries, "ppt/slides/slide2.xml", slide_with(picture(4, "rId2"), "Two"))
    entries += [("ppt/media/image1.png", PNG), ("ppt/media/image2.png", PNG + b"\0"),
                ("ppt/theme/_rels/theme1.xml.rels", rels(("image", "../media/image2.png")))]
    return entries


DIAGRAM_NS = 'xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"'
DRAWING_REL = "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing"
DSP = "http://schemas.microsoft.com/office/drawing/2008/diagram"
CHART_NS = 'xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="' + T + '"'
UNKNOWN_REL = "http://example.com/relationships/retained"
INK_REL = "http://schemas.microsoft.com/office/2011/relationships/ink"
EXT_DATA_REL = f"{T}/package"


def frame(sid, uri, inner):
    return (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="Frame {sid}"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
            '<p:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></p:xfrm>'
            f'<a:graphic><a:graphicData uri="{uri}">{inner}</a:graphicData></a:graphic></p:graphicFrame>')


def rich_clone():
    """A slide with a chart and its workbook, SmartArt, OLE and an unknown part."""
    chart = frame(4, "http://schemas.openxmlformats.org/drawingml/2006/chart",
                  f'<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" r:id="rId2"/>')
    smart = frame(5, "http://schemas.openxmlformats.org/drawingml/2006/diagram",
                  '<dgm:relIds xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" '
                  'r:dm="rId3" r:lo="rId4" r:qs="rId5" r:cs="rId6"/>')
    ole = frame(6, "http://schemas.openxmlformats.org/presentationml/2006/ole",
                '<p:oleObj name="Object" r:id="rId8" imgW="100" imgH="100" progId="Package"><p:embed/></p:oleObj>')
    entries = deck(2, slide_rels=[("chart", "../charts/chart1.xml"), ("diagramData", "../diagrams/data1.xml"),
                                  ("diagramLayout", "../diagrams/layout1.xml"), ("diagramQuickStyle", "../diagrams/quickStyle1.xml"),
                                  ("diagramColors", "../diagrams/colors1.xml"), (DRAWING_REL, "../diagrams/drawing1.xml"),
                                  ("oleObject", "../embeddings/oleObject1.bin"), (UNKNOWN_REL, "../custom/thing1.xml"),
                                  (INK_REL, "../ink/ink1.xml")])
    entries = replace(entries, "ppt/slides/slide1.xml", slide_with(chart + smart + ole, "Rich"))
    entries = replace(entries, "ppt/slides/_rels/slide2.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml")))
    entries += [
        ("ppt/charts/chart1.xml",
         f'{DECL}<c:chartSpace {CHART_NS}><c:chart><c:plotArea><c:layout/><c:areaChart><c:axId val="1"/><c:axId val="2"/></c:areaChart></c:plotArea><c:plotVisOnly val="1"/></c:chart>'
         '<c:externalData r:id="rId1"><c:autoUpdate val="0"/></c:externalData></c:chartSpace>'),
        ("ppt/charts/_rels/chart1.xml.rels", rels(("package", "../embeddings/Workbook1.xlsx"))),
        ("ppt/embeddings/Workbook1.xlsx", b"PK\x05\x06" + b"\0" * 18),
        ("ppt/embeddings/oleObject1.bin", b"\xd0\xcf\x11\xe0OLE"),
        ("ppt/diagrams/data1.xml",
         f'{DECL}<dgm:dataModel {DIAGRAM_NS}><dgm:ptLst><dgm:pt modelId="0" type="doc"/></dgm:ptLst><dgm:bg/><dgm:whole/>'
         '<dgm:extLst><a:ext uri="http://schemas.microsoft.com/office/drawing/2008/diagram">'
         f'<dsp:dataModelExt xmlns:dsp="{DSP}" relId="rId7" minVer="http://schemas.openxmlformats.org/drawingml/2006/diagram"/>'
         '</a:ext></dgm:extLst></dgm:dataModel>'),
        ("ppt/diagrams/layout1.xml",
         f'{DECL}<dgm:layoutDef {DIAGRAM_NS}><dgm:layoutNode/></dgm:layoutDef>'),
        ("ppt/diagrams/quickStyle1.xml",
         f'{DECL}<dgm:styleDef {DIAGRAM_NS} uniqueId="q"><dgm:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></dgm:scene3d>'
         '<dgm:styleLbl name="node0"><dgm:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></dgm:scene3d><dgm:sp3d/></dgm:styleLbl></dgm:styleDef>'),
        ("ppt/diagrams/colors1.xml",
         f'{DECL}<dgm:colorsDef {DIAGRAM_NS} uniqueId="c"><dgm:styleLbl name="node0"/></dgm:colorsDef>'),
        ("ppt/diagrams/drawing1.xml",
         f'{DECL}<dsp:drawing xmlns:dsp="{DSP}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">'
         '<dsp:spTree><dsp:nvGrpSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvGrpSpPr/></dsp:nvGrpSpPr><dsp:grpSpPr/></dsp:spTree></dsp:drawing>'),
        ("ppt/custom/thing1.xml", f'{DECL}<thing xmlns="urn:example:retained"><kept/></thing>'),
        ("ppt/ink/ink1.xml", f'{DECL}<inkml:ink xmlns:inkml="http://www.w3.org/2003/InkML"/>'),
    ]
    entries = with_default(entries, "bin", "application/vnd.openxmlformats-officedocument.oleObject")
    entries = with_default(entries, "xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    return with_override(
        entries,
        ("/ppt/charts/chart1.xml", "application/vnd.openxmlformats-officedocument.drawingml.chart+xml"),
        ("/ppt/diagrams/data1.xml", "application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml"),
        ("/ppt/diagrams/layout1.xml", "application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml"),
        ("/ppt/diagrams/quickStyle1.xml", "application/vnd.openxmlformats-officedocument.drawingml.diagramStyle+xml"),
        ("/ppt/diagrams/colors1.xml", "application/vnd.openxmlformats-officedocument.drawingml.diagramColors+xml"),
        ("/ppt/diagrams/drawing1.xml", "application/vnd.ms-office.drawingml.diagramDrawing+xml"),
        ("/ppt/custom/thing1.xml", "application/vnd.example.retained+xml"),
        ("/ppt/ink/ink1.xml", "application/inkml+xml"))


def absolute_targets():
    entries = deck(2)
    out = []
    for name, data in entries:
        if name == "ppt/_rels/presentation.xml.rels":
            data = data.replace('Target="', 'Target="/ppt/')
        out.append((name, data))
    return out


def high_ids():
    entries = deck(2)
    main = pres(masters='<p:sldMasterId id="4294967295" r:id="rId1"/>', slides=[(256, "rId3"), (2147483647, "rId4")])
    entries = replace(entries, "ppt/presentation.xml", main)
    out = []
    for name, data in entries:
        if name == "ppt/slideMasters/slideMaster1.xml":
            data = data.replace('id="2147483649"', 'id="2147483650"')
        out.append((name, data))
    return out


def unknown_rel():
    entries = deck(1)
    slide_rels = (f'{DECL}<Relationships xmlns="{PKG_RELS}">'
                  f'<Relationship Id="rId1" Type="{T}/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>'
                  f'<Relationship Id="rId7" Type="{UNKNOWN_REL}" Target="../custom/thing1.xml"/>'
                  f'<Relationship Id="foo" Type="{UNKNOWN_REL}" Target="http://example.com/" TargetMode="External"/>'
                  '</Relationships>')
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", slide_rels)
    entries.append(("ppt/custom/thing1.xml", f'{DECL}<thing xmlns="urn:example:retained"/>'))
    return with_override(entries, ("/ppt/custom/thing1.xml", "application/vnd.example.retained+xml"))


def orphan():
    entries = deck(1)
    entries.append(("ppt/custom/orphan1.xml", f'{DECL}<thing xmlns="urn:example:retained"/>'))
    return with_override(entries, ("/ppt/custom/orphan1.xml", "application/vnd.example.retained+xml"))


def conflicting_default():
    entries = deck(1)
    return [(n, d.replace('Extension="png" ContentType="image/png"', 'Extension="png" ContentType="application/x-other"')
             if n == "[Content_Types].xml" else d) for n, d in entries]


def two_masters():
    """Two masters, the first with a spare layout and no slide on it, on themes of their own."""
    entries = deck(1)
    layout2 = (f'{DECL}<p:sldLayout {NS} type="blank"><p:cSld name="Blank"><p:spTree>{TREE}</p:spTree></p:cSld></p:sldLayout>')
    master = lambda ids: (f'{DECL}<p:sldMaster {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>{CLRMAP}'
                          f'<p:sldLayoutIdLst>{ids}</p:sldLayoutIdLst></p:sldMaster>')
    pres_rels = rels(("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"),
                     ("slideMaster", "slideMasters/slideMaster2.xml"))
    main = pres(masters='<p:sldMasterId id="2147483648" r:id="rId1"/><p:sldMasterId id="2147483660" r:id="rId4"/>',
                slides=[(256, "rId3")])
    entries = replace(entries, "ppt/presentation.xml", main)
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", pres_rels)
    entries = replace(entries, "ppt/slideMasters/slideMaster1.xml",
                      master('<p:sldLayoutId id="2147483649" r:id="rId1"/><p:sldLayoutId id="2147483650" r:id="rId3"/>'))
    entries = replace(entries, "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                      rels(("slideLayout", "../slideLayouts/slideLayout2.xml"), ("theme", "../theme/theme1.xml"),
                           ("slideLayout", "../slideLayouts/slideLayout3.xml")))
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml")))
    entries = replace(entries, "ppt/slideLayouts/_rels/slideLayout1.xml.rels", rels(("slideMaster", "../slideMasters/slideMaster2.xml")))
    entries += [("ppt/slideLayouts/slideLayout2.xml", layout2),
                ("ppt/slideLayouts/_rels/slideLayout2.xml.rels", rels(("slideMaster", "../slideMasters/slideMaster1.xml"))),
                ("ppt/slideLayouts/slideLayout3.xml", layout2),
                ("ppt/slideLayouts/_rels/slideLayout3.xml.rels", rels(("slideMaster", "../slideMasters/slideMaster1.xml"))),
                ("ppt/slideMasters/slideMaster2.xml", master('<p:sldLayoutId id="2147483661" r:id="rId1"/>')),
                ("ppt/slideMasters/_rels/slideMaster2.xml.rels",
                 rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("theme", "../theme/theme2.xml"))),
                ("ppt/theme/theme2.xml", THEME)]
    return with_override(entries,
                         ("/ppt/slideLayouts/slideLayout2.xml", f"{PML}.slideLayout+xml"),
                         ("/ppt/slideLayouts/slideLayout3.xml", f"{PML}.slideLayout+xml"),
                         ("/ppt/slideMasters/slideMaster2.xml", f"{PML}.slideMaster+xml"),
                         ("/ppt/theme/theme2.xml", THEME_TYPE))


def shared_theme():
    """One theme under the master and under the notes master, as a deck can arrive."""
    entries = [(n, d) for n, d in noted() if n != "ppt/theme/theme2.xml"]
    entries = [(n, d.replace(f'<Override PartName="/ppt/theme/theme2.xml" ContentType="{THEME_TYPE}"/>', "")
                if n == "[Content_Types].xml" else d) for n, d in entries]
    return replace(entries, "ppt/notesMasters/_rels/notesMaster1.xml.rels", rels(("theme", "../theme/theme1.xml")))


def four_three():
    entries = deck(1)
    out = []
    for name, data in entries:
        if name == "ppt/presentation.xml":
            data = data.replace('<p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>',
                                '<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/><p:notesSz cx="3429000" cy="4572000"/>')
        out.append((name, data))
    return out


TIMING = ('<p:timing><p:tnLst><p:par><p:cTn id="1" dur="indefinite" restart="never" nodeType="tmRoot"><p:childTnLst>'
          '<p:seq concurrent="1" nextAc="seek"><p:cTn id="2" dur="indefinite" nodeType="mainSeq"><p:childTnLst>'
          '<p:par><p:cTn id="3" fill="hold"><p:stCondLst><p:cond delay="indefinite"/></p:stCondLst><p:childTnLst>'
          '<p:par><p:cTn id="4" fill="hold"><p:stCondLst><p:cond delay="0"/></p:stCondLst><p:childTnLst>'
          '<p:par><p:cTn id="5" presetID="1" presetClass="entr" presetSubtype="0" fill="hold" nodeType="clickEffect">'
          '<p:stCondLst><p:cond delay="0"/></p:stCondLst><p:childTnLst>'
          '<p:set><p:cBhvr><p:cTn id="6" dur="1" fill="hold"><p:stCondLst><p:cond delay="0"/></p:stCondLst></p:cTn>'
          '<p:tgtEl><p:spTgt spid="4"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr>'
          '<p:to><p:strVal val="visible"/></p:to></p:set>'
          '</p:childTnLst></p:cTn></p:par></p:childTnLst></p:cTn></p:par></p:childTnLst></p:cTn></p:par></p:childTnLst></p:cTn>'
          '<p:prevCondLst><p:cond evt="onPrev" delay="0"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:prevCondLst>'
          '<p:nextCondLst><p:cond evt="onNext" delay="0"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:nextCondLst>'
          '</p:seq></p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>')


def animated():
    """One slide with one shape, and a timing tree that names it."""
    a16 = ('<a:extLst><a:ext uri="{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}"><a16:creationId '
           'xmlns:a16="http://schemas.microsoft.com/office/drawing/2014/main" id="{AAAAAAAA-0000-4000-8000-000000000001}"/></a:ext></a:extLst>')
    body = sp(4, "Animated", cnvpr_body=a16)
    slide = f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{TITLE % "Animated"}{body}</p:spTree></p:cSld>{TIMING}</p:sld>'
    return replace(deck(1), "ppt/slides/slide1.xml", slide)


def vba():
    """A macro deck whose `bin` Default is VBA's, so dropping VBA has a Default to take."""
    entries = deck(1, main_type="application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
                   parts=[("ppt/vbaProject.bin", b"x")])
    entries = with_default(entries, "bin", "application/vnd.ms-office.vbaProject")
    return replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"),
        ("http://schemas.microsoft.com/office/2006/relationships/vbaProject", "vbaProject.bin")))


def fixtures():
    yield "generated/rr-vba.pptm", zipped(vba()), "open CS-FND-0008"
    yield "generated/rr-animated.pptx", zipped(animated()), "open"
    yield "generated/rr-sections.pptx", zipped(sectioned()), "open"
    yield "generated/rr-one-section.pptx", zipped(one_section()), "open"
    yield "generated/rr-notes.pptx", zipped(noted()), "open"
    yield "generated/rr-shared-media.pptx", zipped(shared_media()), "open"
    yield "generated/rr-rich-clone.pptx", zipped(rich_clone()), "open CS-FND-0007 CS-FND-0010"
    yield "generated/rr-absolute-targets.pptx", zipped(absolute_targets()), "open"
    yield "generated/rr-high-ids.pptx", zipped(high_ids()), "open"
    yield "generated/rr-unknown-rel.pptx", zipped(unknown_rel()), "open CS-FND-0001 CS-FND-0007"
    yield "generated/rr-orphan.pptx", zipped(orphan()), "open CS-FND-0005"
    yield "generated/rr-conflicting-default.pptx", zipped(conflicting_default()), "open"
    yield "generated/rr-two-masters.pptx", zipped(two_masters()), "open"
    yield "generated/rr-shared-theme.pptx", zipped(shared_theme()), "open"
    yield "generated/rr-four-three.pptx", zipped(four_three()), "open"
