#!/usr/bin/env python3
"""Tier 1G fixtures: tables, merges, the tableStyles part, and hostile tables. **Synthetic:**
written to the schema and to MS-OI29500's account of merges, not by PowerPoint, so they show
we follow those and keep what is there, not what Office does with it.

Imported by generate.py, which writes them and lists them in the manifest. The expected
values are in crates/casual-slide-import/tests/table.rs and
crates/casual-slide-export/tests/table.rs.
"""
import hashlib

from generate import DECL, NS, TREE, deck, replace, rels, seeded, strict, zipped

STYLE_ID = "{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}"
INLINE_ID = "{2D9F4B3F-6C7E-4F80-9BAC-1D2E3F4A5B6C}"
UNKNOWN_ID = "{00000000-DEAD-4BEE-8F00-000000000000}"
TABLE_URI = "http://schemas.openxmlformats.org/drawingml/2006/table"
STYLES_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.tableStyles+xml"
DML = "http://schemas.openxmlformats.org/drawingml/2006/main"
A16 = "http://schemas.microsoft.com/office/drawing/2014/main"
SOLID = '<a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>'
EFFECTS = '<a:effectLst><a:glow rad="1000"><a:srgbClr val="00FF00"/></a:glow></a:effectLst>'
EXT = '<a:extLst><a:ext uri="{00000000-0000-0000-0000-00000000000%d}"/></a:extLst>'
BODY = '<a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>%s</a:t></a:r></a:p></a:txBody>'
EMPTY_BODY = "<a:txBody><a:bodyPr/><a:lstStyle/><a:p/></a:txBody>"
CELL3D = ('<a:cell3D prstMaterial="metal"><a:bevel w="1000" h="2000" prst="angle"/>'
          '<a:lightRig rig="flood" dir="t"/>' + EXT % 1 + '</a:cell3D>')
LINE = ('<a:%s w="12700" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:srgbClr val="004B6E"/></a:solidFill>'
        '<a:prstDash val="solid"/><a:round/><a:headEnd type="none" w="med" len="med"/><a:tailEnd type="none" w="med" len="med"/></a:%s>')


def col_id(n):
    return f'<a:extLst><a:ext uri="{{9D8B030D-6E8A-4147-A177-3AD203B41FA5}}"><a16:colId xmlns:a16="{A16}" val="{n}"/></a:ext></a:extLst>'


def row_id(n):
    return f'<a:extLst><a:ext uri="{{0D108BD9-81ED-4DB2-BD59-A6C34878D82A}}"><a16:rowId xmlns:a16="{A16}" val="{n}"/></a:ext></a:extLst>'


def frame(sid, name, tbl):
    return (f'<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{sid}" name="{name}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr>'
            f'<p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="1000000"/></p:xfrm>'
            f'<a:graphic><a:graphicData uri="{TABLE_URI}">{tbl}</a:graphicData></a:graphic></p:graphicFrame>')


def slide_of(*shapes):
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{"".join(shapes)}</p:spTree></p:cSld></p:sld>'


def with_parts(slide, styles=None):
    """A deck holding this slide, and a tableStyles part if given."""
    if styles is None:
        return replace(deck(), "ppt/slides/slide1.xml", slide)
    entries = deck(parts=[("ppt/tableStyles.xml", styles)], types_extra=[("/ppt/tableStyles.xml", STYLES_TYPE)])
    entries = replace(entries, "ppt/slides/slide1.xml", slide)
    return replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"),
        ("slide", "slides/slide1.xml"), ("tableStyles", "tableStyles.xml")))


def every_part(fill_kind):
    """All thirteen part styles, each saying which it is by its own colour."""
    out = []
    names = ["wholeTbl", "band1H", "band2H", "band1V", "band2V", "lastCol", "firstCol", "lastRow",
             "seCell", "swCell", "firstRow", "neCell", "nwCell"]
    for i, name in enumerate(names):
        colour = f"{i + 1:02X}{i + 1:02X}{i + 1:02X}"
        out.append(f'<a:{name}><a:tcTxStyle b="on" i="off"><a:fontRef idx="minor"><a:prstClr val="black"/></a:fontRef>'
                   f'<a:srgbClr val="{colour}"/></a:tcTxStyle><a:tcStyle><a:tcBdr/>'
                   f'<a:fill><a:solidFill><a:srgbClr val="{colour}"/></a:solidFill></a:fill></a:tcStyle></a:{name}>')
    if fill_kind == "rich":
        # The first part again with everything the schema lets it hold.
        out[0] = ('<a:wholeTbl><a:tcTxStyle b="def" i="on"><a:font><a:latin typeface="Arial" panose="020F0502020204030204" pitchFamily="34" charset="0"/>'
                  '<a:ea typeface=""/><a:cs typeface=""/><a:font script="Jpan" typeface="Meiryo"/></a:font><a:schemeClr val="dk1"/>' + EXT % 2 + '</a:tcTxStyle>'
                  '<a:tcStyle><a:tcBdr>'
                  '<a:left><a:ln w="1"><a:solidFill><a:srgbClr val="010101"/></a:solidFill></a:ln></a:left>'
                  '<a:right><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef></a:right>'
                  '<a:top><a:ln w="3"/></a:top><a:bottom><a:ln w="4"/></a:bottom><a:insideH><a:ln w="5"/></a:insideH><a:insideV><a:ln w="6"/></a:insideV>'
                  '<a:tl2br><a:ln w="7"/></a:tl2br><a:tr2bl><a:ln w="8"/></a:tr2bl>' + EXT % 3 + '</a:tcBdr>'
                  '<a:fillRef idx="2"><a:schemeClr val="accent2"/></a:fillRef>' + CELL3D + '</a:tcStyle></a:wholeTbl>')
    return "".join(out)


def style(style_id, name, background=True):
    bg = ('<a:tblBg><a:fill><a:solidFill><a:srgbClr val="ABCDEF"/></a:solidFill></a:fill>'
          '<a:effectRef idx="3"><a:schemeClr val="accent3"/></a:effectRef></a:tblBg>') if background else ""
    return f'<a:tblStyle styleId="{style_id}" styleName="{name}">{bg}{every_part("rich")}{EXT % 4}</a:tblStyle>'


STYLES = (f'{DECL}<a:tblStyleLst xmlns:a="{DML}" def="{STYLE_ID}">'
          + style(STYLE_ID, "Fixture Style")
          # The other arm of each choice in tblBg.
          + f'<a:tblStyle styleId="{INLINE_ID}" styleName="Second"><a:tblBg><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef>'
            f'<a:effect>{EFFECTS}</a:effect></a:tblBg></a:tblStyle>'
          + '</a:tblStyleLst>')


def every_property_table():
    props = (f'<a:tblPr rtl="1" firstRow="1" firstCol="1" lastRow="1" lastCol="1" bandRow="1" bandCol="1">{SOLID}{EFFECTS}'
             f'<a:tableStyleId>{STYLE_ID}</a:tableStyleId>{EXT % 5}</a:tblPr>')
    grid = f'<a:tblGrid><a:gridCol w="2000000">{col_id(11)}</a:gridCol><a:gridCol w="2000000">{col_id(12)}</a:gridCol></a:tblGrid>'
    lines = "".join(LINE % (n, n) for n in ("lnL", "lnR", "lnT", "lnB", "lnTlToBr", "lnBlToTr"))
    full = (f'<a:tc rowSpan="1" gridSpan="1" hMerge="0" vMerge="0" id="cell-1">{BODY % "Full"}'
            f'<a:tcPr marL="91440" marR="1in" marT="45720" marB="2.5cm" vert="vert270" anchor="ctr" anchorCtr="1" horzOverflow="overflow">'
            f'{lines}{CELL3D}{SOLID}<a:headers><a:header>h1</a:header><a:header>h2</a:header></a:headers>{EXT % 6}</a:tcPr>{EXT % 7}</a:tc>')
    plain = f'<a:tc>{BODY % "Plain"}<a:tcPr/></a:tc>'
    return (f'<a:tbl>{props}{grid}<a:tr h="370000">{full}{plain}{row_id(21)}</a:tr>'
            f'<a:tr h="370000">{plain}{plain}{row_id(22)}</a:tr></a:tbl>')


def inline_style_table():
    props = f'<a:tblPr><a:noFill/><a:effectDag><a:cont/></a:effectDag><a:tableStyle styleId="{INLINE_ID}" styleName="Inline">{every_part("plain")}</a:tableStyle></a:tblPr>'
    return f'<a:tbl>{props}<a:tblGrid><a:gridCol w="4000000"/></a:tblGrid><a:tr h="370000"><a:tc>{EMPTY_BODY}</a:tc></a:tr></a:tbl>'


def every_property_deck():
    return with_parts(slide_of(frame(2, "Every Property", every_property_table()),
                               frame(3, "Inline Style", inline_style_table())), STYLES)


def merged_table():
    """3 rows by 4 columns, as MS-OI29500 2.1.1424 d describes: a 2-by-2 merge, a horizontal
    one and a vertical one. Text in the origins, and each covered cell its own."""
    def cell(attrs, text):
        return f'<a:tc{attrs}>{BODY % text}<a:tcPr/></a:tc>'
    grid = "".join('<a:gridCol w="1000000"/>' for _ in range(4))
    r1 = (cell(' gridSpan="2" rowSpan="2"', "Origin 2x2") + cell(' rowSpan="2" hMerge="1"', "c")
          + cell(' gridSpan="2"', "Wide") + cell(' hMerge="1"', "c"))
    r2 = (cell(' gridSpan="2" vMerge="1"', "c") + cell(' hMerge="1" vMerge="1"', "c")
          + cell(' rowSpan="2"', "Tall") + cell("", "Plain"))
    r3 = (cell("", "A") + cell("", "B") + cell(' vMerge="1"', "c") + cell("", "D"))
    props = f'<a:tblPr firstRow="1" bandRow="1"><a:tableStyleId>{STYLE_ID}</a:tableStyleId></a:tblPr>'
    return (f'<a:tbl>{props}<a:tblGrid>{grid}</a:tblGrid><a:tr h="370000">{r1}</a:tr>'
            f'<a:tr h="370000">{r2}</a:tr><a:tr h="370000">{r3}</a:tr></a:tbl>')


def plain_table(style_id):
    props = f'<a:tblPr firstRow="1"><a:tableStyleId>{style_id}</a:tableStyleId></a:tblPr>'
    return f'<a:tbl>{props}<a:tblGrid><a:gridCol w="4000000"/></a:tblGrid><a:tr h="370000"><a:tc>{BODY % "x"}<a:tcPr/></a:tc></a:tr></a:tbl>'



def fixtures():
    """(relative path, bytes, manifest expectation), in a fixed order."""
    entries = every_property_deck()
    yield "generated/table-every-property.pptx", zipped(entries), "open"
    yield "generated/table-strict.pptx", zipped([(n, d) if isinstance(d, bytes) else strict([(n, d)])[0] for n, d in entries]), "open"
    yield "generated/table-merged-cells.pptx", zipped(with_parts(slide_of(frame(2, "Merged", merged_table())), STYLES)), "open"
    # BEH-29: no tableStyles part at all, and a part that does not define the id.
    yield "generated/table-unknown-style.pptx", zipped(with_parts(slide_of(frame(2, "Unknown", plain_table(UNKNOWN_ID))))), "open"
    yield "generated/table-unknown-style-with-list.pptx", zipped(with_parts(slide_of(frame(2, "Unknown", plain_table(UNKNOWN_ID))), STYLES)), "open"
    yield from hostile()


def varied(unit, count, salt):
    """`count` of `unit` broken by comments that differ, so compression cannot make
    the expansion ratio refuse the file before the bound under test."""
    tag = hashlib.sha256(str(salt).encode()).hexdigest()
    return f"<!--{tag}-->" + seeded(unit, count)


def hostile():
    def one(tbl, styles=None):
        return zipped(with_parts(slide_of(frame(2, "t", tbl)), styles))
    grid1 = '<a:tblGrid><a:gridCol w="1"/></a:tblGrid>'
    yield "hostile/table-limit-rows.bin", one(f'<a:tbl>{grid1}' + seeded('<a:tr h="0"/>', 4_097) + '</a:tbl>'), "open"
    yield "hostile/table-limit-columns.bin", one('<a:tbl><a:tblGrid>' + seeded('<a:gridCol w="1"/>', 4_097) + '</a:tblGrid></a:tbl>'), "open"
    # 4096 rows of 257 cells: within the rows bound, and each row within the table's.
    row = "".join(f'<a:tr h="0">{varied("<a:tc/>", 257, i)}</a:tr>' for i in range(4_096))
    yield "hostile/table-limit-cells-in-table.bin", one(f'<a:tbl>{grid1}{row}</a:tbl>'), "open"
    yield "hostile/table-limit-headers.bin", one(f'<a:tbl>{grid1}<a:tr h="0"><a:tc><a:tcPr><a:headers>' + seeded('<a:header>h</a:header>', 1_025) + '</a:headers></a:tcPr></a:tc></a:tr></a:tbl>'), "open"
    many = f'{DECL}<a:tblStyleLst xmlns:a="{DML}" def="{STYLE_ID}">' + seeded(f'<a:tblStyle styleId="{STYLE_ID}" styleName="s"/>', 4_097) + '</a:tblStyleLst>'
    yield "hostile/table-limit-styles.bin", one(plain_table(STYLE_ID), many), "open"
    # Everything reported and nothing refused. The marker must reach no finding.
    mark = "QzMarkerQz"
    grid3 = '<a:tblGrid><a:gridCol w="-5"/><a:gridCol w="QzMarkerQz"/><a:gridCol/></a:tblGrid>'
    c = "<a:tc/>"
    bad = (f'<a:tbl><a:tblPr rtl="{mark}" firstRow="{mark}" unknown="{mark}"><a:tableStyleId>{mark}</a:tableStyleId><a:tableStyleId>{STYLE_ID}</a:tableStyleId><a:noFill/><a:noFill/><a:unknown/></a:tblPr><a:tblPr/>'
           f'{grid3}<a:tblGrid/>'
           # A span past the grid, past the table, of zero, negative, of the largest int, of text.
           f'<a:tr h="-1"><a:tc gridSpan="4"/><a:tc rowSpan="9"/><a:tc gridSpan="2147483647" rowSpan="2147483647"/></a:tr>'
           f'<a:tr h="2147483648"><a:tc gridSpan="0"/><a:tc rowSpan="-3"/><a:tc gridSpan="{mark}" rowSpan="99999999999"/></a:tr>'
           # Too few, too many cells.
           f'<a:tr h="{mark}">{c}</a:tr><a:tr>{c * 5}</a:tr>'
           # Flags with no origin; a covered cell with no flag; merges overlapping.
           f'<a:tr h="0"><a:tc hMerge="1"/><a:tc vMerge="1"/><a:tc hMerge="{mark}" vMerge="{mark}"/></a:tr>'
           f'<a:tr h="0"><a:tc gridSpan="3" rowSpan="2"/>{c}<a:tc gridSpan="1" rowSpan="2" hMerge="1"/></a:tr>'
           f'<a:tr h="0"><a:tc gridSpan="2"/><a:tc hMerge="1"/><a:tc vMerge="1"/></a:tr>'
           # A cell's damage: repeated, unknown, bad values, hostile lines, no bevel.
           f'<a:tr h="0"><a:tc id="{mark}" unknown="{mark}">{EMPTY_BODY}{EMPTY_BODY}<a:tcPr marL="{mark}" marR="99999999999" vert="{mark}" anchor="{mark}" anchorCtr="{mark}" horzOverflow="{mark}" unknown="{mark}">'
           f'<a:lnL w="-1"/><a:lnL/><a:cell3D prstMaterial="{mark}"/><a:noFill/><a:noFill/><a:headers><a:unknown/><a:header><a:b/></a:header></a:headers><a:headers/><a:unknown/></a:tcPr><a:tcPr/><a:unknown/></a:tc>{c}{c}</a:tr>'
           f'</a:tbl><a:tbl/>')
    bad_styles = (f'{DECL}<a:tblStyleLst xmlns:a="{DML}" def="{mark}" unknown="{mark}"><a:unknown/><a:tblStyle/>'
                  f'<a:tblStyle styleId="{STYLE_ID}" styleName="a"><a:wholeTbl><a:tcTxStyle b="{mark}" i="{mark}"><a:font/><a:fontRef/><a:srgbClr val="{mark}"/><a:srgbClr/></a:tcTxStyle><a:tcTxStyle/>'
                  f'<a:tcStyle><a:tcBdr><a:left/><a:left/><a:right><a:unknown/><a:ln/><a:lnRef/></a:right><a:unknown/></a:tcBdr><a:tcBdr/><a:fill/><a:fillRef idx="{mark}"/><a:cell3D/><a:cell3D/><a:unknown/></a:tcStyle><a:unknown/></a:wholeTbl><a:wholeTbl/>'
                  f'<a:tblBg><a:fill><a:unknown/><a:noFill/><a:noFill/></a:fill><a:fillRef/><a:effect><a:unknown/><a:effectLst/><a:effectLst/></a:effect><a:effectRef idx="-1"/><a:unknown/></a:tblBg><a:unknown/></a:tblStyle>'
                  f'<a:tblStyle styleId="{STYLE_ID}" styleName="repeat"/><a:tblStyle styleId="{mark}" styleName="{mark}"><a:tblBg><a:effect/></a:tblBg></a:tblStyle></a:tblStyleLst>')
    yield "hostile/table-bad-structure.bin", one(bad, bad_styles), "open"
    # A tableStyles part that is not one: reported, and the deck opens. One that is not XML is
    # built in crates/casual-slide-export/tests/table.rs, as every "open" deck in the corpus
    # is held to parse part by part.
    wrong = f'{DECL}<a:notAList xmlns:a="{DML}"/>'
    yield "hostile/table-styles-wrong-root.bin", one(plain_table(STYLE_ID), wrong), "open"
