#!/usr/bin/env python3
"""Tier 1F fixtures: text bodies, every property, every enum value, the inheritance chain,
and hostile text. **Synthetic:** written to the schema, not by PowerPoint, so they show we
follow the schema and keep what is there, not what Office does with it.

Imported by generate.py, which writes them and lists them in the manifest. The expected
values are in crates/casual-slide-import/tests/text.rs and
crates/casual-slide-style/tests/text_chain.rs.
"""
import csv
import pathlib
import re

from generate import CLRMAP, DECL, NS, TREE, deck, replace, seeded, strict, zipped

REPO = pathlib.Path(__file__).resolve().parents[2]
PICTURE = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000001"
                        "e221bc330000000049454e44ae426082")
GUID = "{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}"
GRADIENT = ('<a:gradFill><a:gsLst><a:gs pos="0"><a:srgbClr val="FF0000"/></a:gs><a:gs pos="100000"><a:srgbClr val="0000FF"/></a:gs></a:gsLst>'
            '<a:lin ang="0" scaled="0"/></a:gradFill>')
FONT_ATTRS = 'panose="020F0502020204030204" pitchFamily="34" charset="0"'


def shape(sid, name, body, ph=""):
    return (f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvSpPr/><p:nvPr>{ph}</p:nvPr></p:nvSpPr>'
            f'<p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
            f'<p:txBody>{body}</p:txBody></p:sp>')


def slide_of(*shapes):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


ALL_RUN_ATTRS = ('kumimoji="1" lang="en-US" altLang="de-DE" sz="2400" b="1" i="0" u="dbl" strike="sngStrike" kern="1200" '
                 'cap="small" spc="-50" normalizeH="1" baseline="30000" noProof="1" dirty="0" err="1" smtClean="0" smtId="7" bmk="mark"')
ALL_PARAGRAPH_ATTRS = ('marL="342900" marR="10" lvl="1" indent="-342900" algn="just" defTabSz="914400" rtl="1" eaLnBrk="0" '
                       'fontAlgn="ctr" latinLnBrk="1" hangingPunct="0"')
ALL_BODY_ATTRS = ('rot="60000" spcFirstLastPara="1" vertOverflow="clip" horzOverflow="clip" vert="vert270" wrap="none" '
                  'lIns="91440" tIns="1in" rIns="2.5cm" bIns="45720" numCol="2" spcCol="91440" rtlCol="1" fromWordArt="1" '
                  'anchor="ctr" anchorCtr="1" forceAA="1" upright="1" compatLnSpc="1"')
ALL_RUN_CHILDREN = ('<a:ln w="12700"><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill></a:ln>'
                    '<a:solidFill><a:schemeClr val="accent1"><a:lumMod val="75000"/></a:schemeClr></a:solidFill>'
                    '<a:effectLst><a:glow rad="1000"><a:srgbClr val="FF0000"/></a:glow></a:effectLst>'
                    '<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>'
                    '<a:uLnTx/><a:uFillTx/>'
                    f'<a:latin typeface="Arial" {FONT_ATTRS}/><a:ea typeface="+mn-ea"/><a:cs typeface="Arial"/><a:sym typeface="Wingdings"/>'
                    '<a:hlinkClick r:id="rId2" tooltip="Go" tgtFrame="_blank" history="0" highlightClick="1" endSnd="1" invalidUrl="http://x.invalid/"/>'
                    '<a:hlinkMouseOver r:id="" action="ppaction://noaction"/>'
                    '<a:rtl val="1"/>'
                    '<a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000001}"/></a:extLst>')
ALL_PARAGRAPH_CHILDREN = ('<a:lnSpc><a:spcPct val="90000"/></a:lnSpc><a:spcBef><a:spcPts val="1000"/></a:spcBef><a:spcAft><a:spcPct val="50%"/></a:spcAft>'
                          '<a:buClr><a:schemeClr val="tx1"/></a:buClr><a:buSzPct val="150%"/><a:buFont typeface="Arial" pitchFamily="34" charset="0"/>'
                          '<a:buChar char="&#8226;"/><a:tabLst><a:tab pos="914400" algn="l"/><a:tab pos="1in" algn="dec"/></a:tabLst>'
                          f'<a:defRPr {ALL_RUN_ATTRS}/><a:extLst><a:ext uri="{{00000000-0000-0000-0000-000000000002}}"/></a:extLst>')


def full_slide():
    every = (f'<a:bodyPr {ALL_BODY_ATTRS}><a:prstTxWarp prst="textPlain"><a:avLst/></a:prstTxWarp>'
             '<a:normAutofit fontScale="62.5%" lnSpcReduction="10000"/>'
             '<a:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></a:scene3d><a:sp3d/>'
             '<a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000003}"/></a:extLst></a:bodyPr>'
             f'<a:lstStyle><a:defPPr algn="r"/>'
             + "".join(f'<a:lvl{i}pPr marL="{i * 1000}"><a:defRPr sz="{1000 + i * 100}"/></a:lvl{i}pPr>' for i in range(1, 10))
             + '<a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000004}"/></a:extLst></a:lstStyle>'
             f'<a:p><a:pPr {ALL_PARAGRAPH_ATTRS}>{ALL_PARAGRAPH_CHILDREN}</a:pPr>'
             f'<a:r><a:rPr {ALL_RUN_ATTRS}>{ALL_RUN_CHILDREN}</a:rPr><a:t>Every property</a:t></a:r>'
             f'<a:br><a:rPr sz="1100"/></a:br>'
             f'<a:fld id="{GUID}" type="slidenum"><a:rPr sz="1200"/><a:pPr algn="ctr"/><a:t>1</a:t></a:fld>'
             '<a:endParaRPr sz="1300" dirty="0"/></a:p>')
    autofits = [("No Autofit", '<a:noAutofit/>'), ("Shape Autofit", '<a:spAutoFit/>'), ("Bare Autofit", '<a:normAutofit/>'),
                ("Decimal Autofit", '<a:normAutofit fontScale="92500" lnSpcReduction="20000"/>'),
                ("Percent Autofit", '<a:normAutofit fontScale="62.5%" lnSpcReduction="10%"/>')]
    fits = [shape(10 + i, name, f'<a:bodyPr>{fit}</a:bodyPr><a:p><a:r><a:t>{name}</a:t></a:r></a:p>') for i, (name, fit) in enumerate(autofits)]
    fills = ('<a:noFill/>', GRADIENT, '<a:blipFill><a:blip r:embed="rId3"/><a:stretch><a:fillRect/></a:stretch></a:blipFill>',
             '<a:pattFill prst="pct5"/>', '<a:grpFill/>')
    fill_runs = "".join(f'<a:r><a:rPr>{f}</a:rPr><a:t>Fill {i}</a:t></a:r>' for i, f in enumerate(fills))
    under = ('<a:r><a:rPr><a:effectDag/><a:uLn w="9525"/><a:uFill><a:solidFill><a:srgbClr val="123456"/></a:solidFill></a:uFill></a:rPr><a:t>Underline</a:t></a:r>')
    bullets = ['<a:buNone/>', '<a:buAutoNum type="arabicPeriod" startAt="3"/>', '<a:buChar char="-"/>',
               '<a:buBlip><a:blip r:embed="rId3"/></a:buBlip>', '<a:buClrTx/><a:buSzTx/><a:buFontTx/><a:buAutoNum type="romanLcPeriod"/>',
               '<a:buSzPts val="1400"/>', '<a:buSzPct val="100%"/>']
    bullet_paragraphs = "".join(f'<a:p><a:pPr>{b}</a:pPr><a:r><a:t>Bullet {i}</a:t></a:r></a:p>' for i, b in enumerate(bullets))
    equation = ('<a:p><a:r><a:t>Before</a:t></a:r>'
                '<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:a14="http://schemas.microsoft.com/office/drawing/2010/main">'
                '<mc:Choice Requires="a14"><a14:m/></mc:Choice><mc:Fallback><a:r><a:t>x</a:t></a:r></mc:Fallback></mc:AlternateContent>'
                '<a:r><a:t>After</a:t></a:r></a:p>')
    return slide_of(
        shape(2, "Every Property", every),
        *fits,
        shape(20, "Fills", f'<a:bodyPr/><a:lstStyle/><a:p>{fill_runs}{under}</a:p>'),
        shape(21, "Bullets", f'<a:bodyPr/><a:lstStyle/>{bullet_paragraphs}'),
        shape(22, "Whitespace", '<a:bodyPr/><a:p><a:r><a:t>  two &amp; &lt;spaced&gt; </a:t></a:r><a:r><a:t/></a:r></a:p><a:p/>'),
        shape(23, "Equation", f'<a:bodyPr/>{equation}'))


def full_deck():
    style = f'<p:defaultTextStyle><a:defPPr><a:defRPr lang="en-US"/></a:defPPr><a:lvl1pPr marL="0"><a:defRPr sz="1800"/></a:lvl1pPr></p:defaultTextStyle>'
    presentation = (f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>'
                    '<p:sldIdLst><p:sldId id="256" r:id="rId3"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>'
                    f'{style}</p:presentation>')
    entries = deck(presentation=presentation,
                   slide_rels=[("hyperlink", "https://example.invalid/", "External"), ("image", "../media/image1.png")],
                   parts=[("ppt/media/image1.png", PICTURE)])
    return replace(entries, "ppt/slides/slide1.xml", full_slide())


def checklist_values():
    """Every enum value in TEXT, by simple type, as the checklist has them."""
    values = {}
    with open(REPO / "docs/spec/checklist.tsv", encoding="utf8") as source:
        for row in csv.DictReader(source, delimiter="\t"):
            if row["area"] == "TEXT" and row["kind"] == "value":
                values.setdefault(row["owner"], []).append(row["name"].removeprefix("="))
    return values


def enum_slide():
    v = checklist_values()
    para = lambda attrs, inner="": f'<a:p><a:pPr {attrs}>{inner}</a:pPr><a:endParaRPr/></a:p>' if attrs else f'<a:p><a:pPr>{inner}</a:pPr></a:p>'
    run = lambda attrs: f'<a:p><a:r><a:rPr {attrs}/><a:t>x</a:t></a:r></a:p>'
    bodies = []
    for attr, typ in (("anchor", "ST_TextAnchoringType"), ("vertOverflow", "ST_TextVertOverflowType"),
                      ("horzOverflow", "ST_TextHorzOverflowType"), ("vert", "ST_TextVerticalType"), ("wrap", "ST_TextWrappingType")):
        bodies += [f'<a:bodyPr {attr}="{x}"/><a:p/>' for x in v[typ]]
    paragraphs = []
    paragraphs += [para(f'algn="{x}"') for x in v["ST_TextAlignType"]]
    paragraphs += [para(f'fontAlgn="{x}"') for x in v["ST_TextFontAlignType"]]
    paragraphs += [para("", f'<a:buAutoNum type="{x}"/>') for x in v["ST_TextAutonumberScheme"]]
    paragraphs += [para("", f'<a:tabLst><a:tab pos="0" algn="{x}"/></a:tabLst>') for x in v["ST_TextTabAlignType"]]
    paragraphs += [run(f'u="{x}"') for x in v["ST_TextUnderlineType"]]
    paragraphs += [run(f'strike="{x}"') for x in v["ST_TextStrikeType"]]
    paragraphs += [run(f'cap="{x}"') for x in v["ST_TextCapsType"]]
    shapes = [shape(2, "Values", f'<a:bodyPr/>{"".join(paragraphs)}')]
    shapes += [shape(10 + i, f"Body {i}", b) for i, b in enumerate(bodies)]
    return slide_of(*shapes)


def level(prefix, tag, size, attrs="", body=""):
    return f'<a:lvl{tag}pPr {attrs}>{body}<a:defRPr sz="{size}" {prefix}/></a:lvl{tag}pPr>'


def chain_deck():
    """Each level states the size and one property nothing nearer states, at levels 1 and 2,
    so a read says where it came from. See text_chain.rs for the table."""
    default = ('<p:defaultTextStyle><a:defPPr algn="thaiDist"/>'
               + level('kern="1"', 1, 1000, 'indent="1"')
               + level('kern="2"', 2, 1001, 'indent="2"')
               + '</p:defaultTextStyle>')
    presentation = (f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>'
                    '<p:sldIdLst><p:sldId id="256" r:id="rId3"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>'
                    f'{default}</p:presentation>')
    styles = ('<p:txStyles>'
              + '<p:titleStyle>' + level('b="1"', 1, 2000, 'marR="1"') + level('b="0"', 2, 2001, 'marR="2"') + '</p:titleStyle>'
              + '<p:bodyStyle>' + level('i="1"', 1, 2100, 'marR="3"') + level('i="0"', 2, 2101, 'marR="4"') + '</p:bodyStyle>'
              + '<p:otherStyle>' + level('u="sng"', 1, 2200, 'marR="5"') + level('u="dbl"', 2, 2201, 'marR="6"') + '</p:otherStyle>'
              + '</p:txStyles>')
    master_body = ('<a:bodyPr wrap="none" anchor="b" lIns="11"/><a:lstStyle>'
                   + level('spc="1"', 1, 3000, 'marL="1"') + level('spc="2"', 2, 3001, 'marL="2"') + '</a:lstStyle><a:p/>')
    master = (f'{DECL}<p:sldMaster {NS}><p:cSld><p:spTree>{TREE}'
              + shape(2, "Master Body", master_body, '<p:ph type="body" idx="1"/>')
              + f'</p:spTree></p:cSld>{CLRMAP}<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>{styles}</p:sldMaster>')
    layout_body = ('<a:bodyPr anchor="ctr" tIns="22"/><a:lstStyle>'
                   + level('cap="all"', 1, 4000, 'defTabSz="1"') + level('cap="small"', 2, 4001, 'defTabSz="2"') + '</a:lstStyle><a:p/>')
    layout = (f'{DECL}<p:sldLayout {NS}><p:cSld><p:spTree>{TREE}'
              + shape(2, "Layout Body", layout_body, '<p:ph type="body" idx="1"/>')
              + '</p:spTree></p:cSld></p:sldLayout>')
    own = ('<a:bodyPr rIns="33"/><a:lstStyle>'
           + level('strike="sngStrike"', 1, 5000, 'fontAlgn="t"') + level('strike="dblStrike"', 2, 5001, 'fontAlgn="b"') + '</a:lstStyle>'
           # A run stating its size; one stating nothing; a paragraph at level 2; `pPr/defRPr`, ignored.
           '<a:p><a:r><a:rPr sz="6000"/><a:t>a</a:t></a:r><a:r><a:t>b</a:t></a:r></a:p>'
           '<a:p><a:pPr lvl="1" algn="r"/><a:r><a:t>c</a:t></a:r></a:p>'
           '<a:p><a:pPr><a:defRPr sz="7000"/></a:pPr><a:r><a:t>d</a:t></a:r></a:p>')
    slide = slide_of(
        shape(2, "Inheriting", own, '<p:ph type="body" idx="1"/>'),
        shape(3, "Free Text", '<a:bodyPr/><a:p><a:r><a:t>e</a:t></a:r></a:p>'),
        shape(4, "Bare Placeholder", '<a:bodyPr/><a:p><a:r><a:t>f</a:t></a:r></a:p>', '<p:ph type="body" idx="1"/>'))
    entries = deck(presentation=presentation)
    entries = replace(entries, "ppt/slideMasters/slideMaster1.xml", master)
    entries = replace(entries, "ppt/slideLayouts/slideLayout1.xml", layout)
    return replace(entries, "ppt/slides/slide1.xml", slide)


def hostile():
    def with_slide(*shapes):
        return zipped(replace(deck(), "ppt/slides/slide1.xml", slide_of(*shapes)))
    yield "hostile/text-limit-paragraphs-in-body.bin", with_slide(shape(2, "t", "<a:bodyPr/>" + seeded("<a:p/>", 100_001))), "open"
    yield "hostile/text-limit-runs-in-paragraph.bin", with_slide(shape(2, "t", "<a:bodyPr/><a:p>" + seeded("<a:br/>", 100_001) + "</a:p>")), "open"
    # Each body is within the per-body bound; together they cross the per-part one.
    third = "<a:bodyPr/>" + seeded("<a:p/>", 90_000)
    yield "hostile/text-limit-paragraphs-in-part.bin", with_slide(*[shape(2 + i, "t", third) for i in range(3)]), "open"
    yield "hostile/text-limit-tab-stops.bin", with_slide(shape(2, "t", '<a:bodyPr/><a:p><a:pPr><a:tabLst>' + seeded('<a:tab/>', 1_025) + '</a:tabLst></a:pPr></a:p>')), "open"
    # Every value unusable, every required thing missing, everything repeated. The marker must
    # reach no finding: it stands where document text can stand.
    mark = "QzMarkerQz"
    bad_values = (f'<a:bodyPr rot="{mark}" vert="{mark}" wrap="{mark}" anchor="{mark}" lIns="{mark}" tIns="99999999999" numCol="17" spcCol="-1" '
                  f'spcFirstLastPara="{mark}" unknown="{mark}"><a:normAutofit fontScale="{mark}" lnSpcReduction="-1"/><a:noAutofit/>'
                  f'<a:prstTxWarp/><a:prstTxWarp prst="{mark}"/></a:bodyPr><a:bodyPr/>'
                  f'<a:lstStyle><a:lvl1pPr marL="-5" lvl="9" algn="{mark}"/><a:lvl1pPr/><a:lvl10pPr/><a:defPPr/><a:defPPr/></a:lstStyle><a:lstStyle/>'
                  f'<a:p><a:pPr marL="99999999" indent="{mark}" defTabSz="{mark}" fontAlgn="{mark}"><a:lnSpc/><a:lnSpc><a:spcPct val="{mark}"/></a:lnSpc>'
                  f'<a:spcBef><a:spcPts val="999999"/><a:spcPct val="1"/></a:spcBef><a:spcAft><a:spcPct val="-1"/></a:spcAft>'
                  f'<a:buClr/><a:buClrTx/><a:buSzPct val="1"/><a:buSzPts/><a:buFont/><a:buFontTx/><a:buAutoNum type="{mark}" startAt="0"/><a:buNone/>'
                  f'<a:tabLst><a:tab pos="{mark}" algn="{mark}"/><a:unknown/></a:tabLst><a:defRPr sz="1"/><a:defRPr/></a:pPr><a:pPr/>'
                  f'<a:r><a:rPr sz="{mark}" b="{mark}" u="{mark}" strike="{mark}" kern="-1" cap="{mark}" spc="{mark}" baseline="{mark}" smtId="-1" unknown="{mark}">'
                  f'<a:solidFill/><a:noFill/><a:highlight/><a:latin/><a:ea panose="{mark}" typeface="{mark}"/><a:rtl val="{mark}"/>'
                  f'<a:hlinkClick r:id="rId{mark}" tooltip="{mark}"/><a:uFill><a:gradFill/><a:solidFill/></a:uFill><a:unknown/></a:rPr>'
                  f'<a:t>{mark}<a:b/></a:t><a:t>{mark}</a:t></a:r>'
                  f'<a:r/><a:r><a:t>{mark}</a:t></a:r>'
                  f'<a:fld id="{mark}" type="{mark}"><a:t>{mark}</a:t></a:fld><a:fld/>'
                  f'<a:br unknown="{mark}"><a:rPr/><a:rPr/></a:br><a:unknown/>'
                  f'<a:endParaRPr/><a:endParaRPr/></a:p><a:unknown/>')
    yield "hostile/text-bad-values.bin", with_slide(shape(2, "t", bad_values), shape(3, "t", "<a:p/>"), shape(4, "t", "<a:bodyPr/>")), "open"


def strict_percent(xml):
    """Strict has only the `N%` form of a percentage."""
    def spelled(match):
        return f'{match.group(1)}"{int(match.group(2)) / 1000:g}%"'
    return re.sub(r'((?:baseline=|fontScale=|lnSpcReduction=|<a:gs pos=|<a:spcPct val=|<a:buSzPct val=|<a:lumMod val=))"([0-9]+)"', spelled, xml)


def fixtures():
    """(relative path, bytes, manifest expectation), in a fixed order."""
    entries = full_deck()
    yield "generated/text-every-property.pptx", zipped(entries), "open"
    yield "generated/text-strict.pptx", zipped([(n, d) if isinstance(d, bytes) else strict([(n, strict_percent(d))])[0] for n, d in entries]), "open"
    yield "generated/text-every-enum-value.pptx", zipped(replace(deck(), "ppt/slides/slide1.xml", enum_slide())), "open"
    yield "generated/text-inheritance.pptx", zipped(chain_deck()), "open"
    yield from hostile()
