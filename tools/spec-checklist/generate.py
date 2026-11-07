#!/usr/bin/env python3
"""Generate the spec checklist: every tag, property and enum value in the schemas.

The "what should exist" side comes from the vendored standards rather than from
anybody's recollection (the idea is OpenCalc's tools/fidelity-audit). Each row is
assigned to an area, and so to a roadmap phase. A type that no rule claims fails
generation, which is what keeps the roadmap complete as schemas are added.

    python3 tools/spec-checklist/generate.py          # rewrite docs/spec/
    python3 tools/spec-checklist/generate.py --check  # fail if docs/spec/ is stale

Status columns in docs/spec/checklist.tsv are hand- or tool-maintained and are
kept across regeneration, keyed by row id.
"""
import json
import pathlib
import re
import sys
import xml.etree.ElementTree as ET
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parents[2]
XS = "{http://www.w3.org/2001/XMLSchema}"
OUT = ROOT / "docs/spec"
STATUS = ("M", "I", "E", "R", "U")  # model, import, export, render, UI
COLUMNS = ("id", "area", "owner", "kind", "name", "type", "use", "default", "restriction") + STATUS

# code: (title, phase-1 tier, render phase, edit phase, level)
# Level: Edit = modelled, drawn, editable. Render = modelled and drawn. Retain = kept
# verbatim and named in the report. Every level is imported and exported in Phase 1.
AREAS = {
    "PKG": ("Package, relationships, properties", "1A", "-", "10", "Edit"),
    "DSIG": ("Digital signatures", "1A", "-", "10", "Retain"),
    "PRES": ("Presentation part and its properties", "1B", "2A", "4", "Edit"),
    "VIEW": ("Saved view state", "1B", "-", "3", "Edit"),
    "COLOR": ("Colour model and transforms", "1C", "2A", "4", "Edit"),
    "THEME": ("Themes, format schemes, colour maps", "1C", "2A", "7", "Edit"),
    "SLIDE": ("Slides, layouts, masters, notes, handouts", "1D", "2A", "4", "Edit"),
    "SHAPE": ("Shape tree, identity, locks, links", "1D", "2B", "4", "Edit"),
    "GEOM": ("Preset and custom geometry", "1E", "2B", "4", "Edit"),
    "FILL": ("Fills", "1E", "2B", "4", "Edit"),
    "LINE": ("Lines and arrowheads", "1E", "2B", "4", "Edit"),
    "EFFECT": ("Effects and blip effects", "1E", "2C", "4", "Edit"),
    "D3": ("3D scene and shape properties", "1E", "2C", "6", "Edit"),
    "TEXT": ("Text bodies, paragraphs, runs, lists", "1F", "2D", "5", "Edit"),
    "TABLE": ("Tables and table styles", "1G", "2E", "6", "Edit"),
    "PIC": ("Pictures and image references", "1H", "2E", "4", "Edit"),
    "MEDIA": ("Audio and video", "1H", "2E", "6", "Edit"),
    "TIMING": ("Animations and timing tree", "1I", "8", "8", "Edit"),
    "TRANS": ("Slide transitions", "1I", "8", "8", "Edit"),
    "CHART": ("Charts", "1J", "2F", "6", "Edit"),
    "CHARTEX": ("Chart extensions (2016+ types)", "1J", "2F", "6", "Edit"),
    "DGM": ("SmartArt diagrams", "1K", "2F", "6", "Edit"),
    "MATH": ("Equations (OMML)", "1K", "2F", "6", "Edit"),
    "INK": ("Ink", "1K", "2F", "6", "Edit"),
    "OLE": ("OLE, ActiveX, embedded objects", "1L", "2F", "-", "Retain"),
    "VML": ("Legacy VML", "1L", "2F", "-", "Retain"),
    "MODEL3D": ("3D models", "1L", "2F", "10", "Edit"),
    "COMMENT": ("Comments, legacy and modern", "1M", "-", "9", "Edit"),
    "REVIEW": ("Change tracking and revision info", "1M", "-", "9", "Retain"),
    "EXT": ("PowerPoint extensions: sections, zoom, classification", "1M", "2F", "8", "Edit"),
    "DESIGN": ("Designer metadata", "1M", "-", "-", "Retain"),
    "CUSTOM": ("Tags, customer data, custom XML, app data", "1M", "-", "-", "Retain"),
    "SIMPLE": ("Shared value types", "1A", "-", "-", "Edit"),
}

# Rules are (file glob name, regex on the type's name, area). First match wins.
PML = [
    (r"Transition|^ST_TransitionSpeed|^ST_TransitionSideDirectionType|^ST_TransitionCornerDirectionType|^ST_TransitionInOutDirectionType|^ST_TransitionEightDirectionType|^ST_Direction$", "TRANS"),
    (r"^CT_TL|^ST_TL|^CT_TimeNodeList|^CT_BuildList|^CT_SlideTiming|^AG_TLBuild|^ST_Iterate|^ST_IterateType", "TIMING"),
    (r"^CT_CommentAuthor|^CT_Comment", "COMMENT"),
    (r"Ole|^CT_Control", "OLE"),
    (r"^CT_(Customer|Tags|String|TagList)", "CUSTOM"),
    (r"^CT_(Normal|Common|NotesText|Outline|SlideSorter|Guide|SlideView|NotesView|View|CommonSlideView|CommonView)|^ST_(SplitterBarState|ViewType|Direction|OleObjectFollowColorScheme)", "VIEW"),
    (r"^CT_(Slide|Notes|Handout|Background|CommonSlide|HeaderFooter|Placeholder|SlideLayout|SlideMaster|Rel)|^EG_(TopLevelSlide|ChildSlide|Background)|^AG_ChildSlide|^ST_(Placeholder|SlideLayout|SlideId|SlideMaster|SlideLayoutId)", "SLIDE"),
    (r"^CT_(ApplicationNonVisual|Shape|Connector|Picture|GraphicalObjectFrame|GroupShape)", "SHAPE"),
    (r"^CT_Extension|^EG_ExtensionList|^CT_Empty|^CT_IndexRange", "PRES"),
    (r"^(CT_CustomShow|CT_CustomShowId|CT_CustomShowList|CT_EmbeddedFontDataId|CT_EmbeddedFontList|CT_EmbeddedFontListEntry|CT_HtmlPublishProperties|CT_Kinsoku|CT_ModifyVerifier|CT_PhotoAlbum|CT_Presentation|CT_PresentationProperties|CT_PrintProperties|CT_ShowInfoBrowse|CT_ShowInfoKiosk|CT_ShowProperties|CT_SmartTags|CT_WebProperties|EG_ShowType|EG_SlideListChoice|ST_BookmarkIdSeed|ST_Index|ST_Name|ST_PhotoAlbumFrameShape|ST_PhotoAlbumLayout|ST_PrintColorMode|ST_PrintWhat|ST_SlideSizeCoordinate|ST_SlideSizeType|ST_WebColorType|ST_WebEncoding|ST_WebScreenSize|cmAuthorLst|cmLst|handoutMaster|notes|notesMaster|oleObj|presentation|presentationPr|sld|sldLayout|sldMaster|sldSyncPr|tagLst|viewPr)$", "PRES"),
]
DML = [
    (r"Audio|Video|QuickTime|EG_Media|WAV", "MEDIA"),
    (r"^CT_(ColorScheme|CustomColor|SupplementalFont|FontCollection|EffectStyleItem|FontScheme|FillStyleList|LineStyleList|EffectStyleList|BackgroundFillStyleList|StyleMatrix|BaseStyles|OfficeStyleSheet|ObjectStyleDefaults|DefaultShapeDefinition|ColorMapping|ColorSchemeAndMapping|ColorSchemeList|ClipboardStyleSheet|EmptyElement|ShapeStyle|FontReference|EffectReference)|^EG_Themeable|^ST_(ColorSchemeIndex|StyleMatrixColumnIndex|FontCollectionIndex|ThemeColor|OnOffStyleType)", "THEME"),
    (r"Transform$|^CT_(ScRgb|SRgb|Hsl|System|Scheme|Preset)Color|^CT_Color|^EG_Color|^ST_(SchemeColorVal|SystemColorVal|PresetColorVal|HexColorRGB)", "COLOR"),
    (r"^CT_Table|^CT_Cell3D|^CT_Headers|^CT_ThemeableLineStyle|^ST_Table", "TABLE"),
    (r"^CT_(Camera|LightRig|Scene3D|Backdrop|Bevel|Shape3D|FlatText|Point3D|Vector3D|SphereCoords)|^EG_Text3D|^ST_(Preset(Camera|Material)|LightRig|Bevel|Fov)", "D3"),
    (r"Effect|^ST_(Blend|PresetShadow|RectAlignment|EffectContainer)", "EFFECT"),
    (r"^CT_(NoFill|SolidColorFill|LinearShade|PathShade|Gradient|TileInfo|StretchInfo|PatternFill|GroupFill|FillProperties|BlipFill|BackgroundFormatting|WholeE2oFormatting)|^EG_(Shade|FillMode|FillProperties)|^ST_(PathShade|PresetPattern|TileFlip)", "FILL"),
    (r"^CT_(Blip|RelativeRect)|^AG_Blob|^ST_BlipCompression|Picture", "PIC"),
    (r"Geom|^CT_(AdjPoint|XYAdjust|PolarAdjust|ConnectionSite|AdjustHandle|Connection$|Path2D|PresetTextShape|PresetGeometry)|^ST_(ShapeType|TextShapeType|Geom|Path|AdjCoordinate|AdjAngle)", "GEOM"),
    (r"^CT_(LineEnd|LineJoin|PresetLineDash|DashStop|LineProperties)|^EG_Line|^ST_(LineEnd|PresetLineDash|LineCap|PenAlignment|CompoundLine|LineWidth)", "LINE"),
    (r"Text|Autofit|^EG_Text|^CT_(RegularTextRun)|^ST_(Font|Anchor|Vertical|Horizontal)", "TEXT"),
    (r"Animation|Build|^ST_(Dgm|Chart|Animation)", "TIMING"),
    (r"^(AG_Locking|CT_Angle|CT_Boolean|CT_ConnectorLocking|CT_ContentPartLocking|CT_FixedPercentage|CT_GraphicalObject|CT_GraphicalObjectData|CT_GraphicalObjectFrameLocking|CT_GroupLocking|CT_GroupShapeProperties|CT_GroupTransform2D|CT_GvmlConnector|CT_GvmlConnectorNonVisual|CT_GvmlGraphicFrameNonVisual|CT_GvmlGraphicalObjectFrame|CT_GvmlGroupShape|CT_GvmlGroupShapeNonVisual|CT_GvmlShape|CT_GvmlShapeNonVisual|CT_GvmlUseShapeRectangle|CT_Hyperlink|CT_NonVisualConnectorProperties|CT_NonVisualContentPartProperties|CT_NonVisualDrawingProps|CT_NonVisualDrawingShapeProps|CT_NonVisualGraphicFrameProperties|CT_NonVisualGroupDrawingShapeProps|CT_OfficeArtExtension|CT_OfficeArtExtensionList|CT_Percentage|CT_Point2D|CT_PositiveFixedAngle|CT_PositiveFixedPercentage|CT_PositivePercentage|CT_PositiveSize2D|CT_Ratio|CT_Scale2D|CT_ShapeLocking|CT_ShapeProperties|CT_Transform2D|EG_OfficeArtExtensionList|ST_Angle|ST_BlackWhiteMode|ST_Coordinate|ST_Coordinate32|ST_Coordinate32Unqualified|ST_CoordinateUnqualified|ST_DrawingElementId|ST_FOVAngle|ST_FixedAngle|ST_FixedPercentage|ST_FixedPercentageDecimal|ST_Percentage|ST_PercentageDecimal|ST_PitchFamily|ST_PositiveCoordinate|ST_PositiveCoordinate32|ST_PositiveFixedAngle|ST_PositiveFixedPercentage|ST_PositiveFixedPercentageDecimal|ST_PositivePercentage|ST_PositivePercentageDecimal|ST_ShapeID|blip|graphic|tbl|tblStyleLst|theme|themeManager|themeOverride|videoFile)$", "SHAPE"),
]
WHOLE_FILE = {
    "pic": "PIC", "lc": "SHAPE", "c": "CHART", "cdr": "CHART", "dgm": "DGM", "m": "MATH",
    "st": "SIMPLE", "r": "SIMPLE", "ep": "PKG", "cp": "PKG", "vt": "PKG", "ds": "CUSTOM",
    "sl": "CUSTOM", "v": "VML", "o": "VML", "pvml": "VML",
    "ct": "PKG", "rel": "PKG", "core": "PKG", "dsig": "DSIG",
}
OOXML = {
    "pml": "p", "dml-main": "a", "dml-picture": "pic", "dml-lockedCanvas": "lc", "dml-chart": "c",
    "dml-chartDrawing": "cdr", "dml-diagram": "dgm", "shared-math": "m",
    "shared-commonSimpleTypes": "st", "shared-relationshipReference": "r",
    "shared-documentPropertiesExtended": "ep", "shared-documentPropertiesCustom": "cp",
    "shared-documentPropertiesVariantTypes": "vt", "shared-customXmlDataProperties": "ds",
    "shared-customXmlSchemaProperties": "sl", "vml-main": "v", "vml-officeDrawing": "o",
    "vml-presentationDrawing": "pvml",
}
OPC = {"opc-contentTypes": "ct", "opc-relationships": "rel", "opc-coreProperties": "core", "opc-digSig": "dsig"}

# Extension namespaces, by the part of the URI after `office/` (or `ink/`).
EXT_AREA = {
    "powerpoint/2010/main": "EXT", "powerpoint/2012/main": "EXT", "powerpoint/2013/main/command": "REVIEW",
    "powerpoint/2014/inkAction": "INK", "powerpoint/2015/09/main": "TRANS", "powerpoint/2015/10/main": "REVIEW",
    "powerpoint/2015/main": "DESIGN", "powerpoint/2016/6/main": "EXT", "powerpoint/2016/sectionzoom": "EXT",
    "powerpoint/2016/slidezoom": "EXT", "powerpoint/2016/summaryzoom": "EXT", "powerpoint/2017/10/main": "EXT",
    "powerpoint/2017/3/main": "MEDIA", "powerpoint/2018/4/main": "EXT", "powerpoint/2018/8/main": "COMMENT",
    "powerpoint/2019/9/main/command": "COMMENT", "powerpoint/2020/02/main": "DESIGN",
    "powerpoint/2022/03/main": "COMMENT", "powerpoint/2022/06/main/command": "COMMENT",
    "powerpoint/2022/08/main": "COMMENT", "powerpoint/2023/02/main": "SLIDE",
    "drawing/2010/main": "EFFECT", "drawing/2010/chartDrawing": "CHART", "drawing/2010/diagram": "DGM",
    "drawing/2010/picture": "PIC", "drawing/2008/diagram": "DGM", "drawing/2007/8/2/chart": "CHART",
    "drawing/2012/chart": "CHART", "drawing/2012/chartStyle": "CHART", "drawing/2012/main": "SHAPE",
    "drawing/2013/main/command": "REVIEW", "drawing/2014/chart": "CHART", "drawing/2014/chart/ac": "CHART",
    "drawing/2014/chartex": "CHARTEX", "drawing/2014/main": "SHAPE", "drawing/2015/06/chart": "CHART",
    "drawing/2016/SVG/main": "PIC", "drawing/2016/11/main": "SHAPE", "drawing/2016/11/diagram": "DGM",
    "drawing/2016/12/diagram": "DGM", "drawing/2016/ink": "INK", "drawing/2017/model3d": "MODEL3D",
    "drawing/2017/03/chart": "CHART", "drawing/2017/decorative": "SHAPE", "drawing/2018/hyperlinkcolor": "SHAPE",
    "drawing/2018/animation": "TIMING", "drawing/2018/animation/model3d": "MODEL3D",
    "drawing/2018/sketchyshapes": "LINE", "drawing/2020/classificationShape": "EXT",
    "drawing/2021/scriptlink": "CUSTOM", "drawing/2021/livefeed": "CUSTOM", "drawing/2022/imageformula": "PIC",
    "thememl/2012/main": "THEME", "ink/2010/main": "INK", "word/2020/oembed": "MEDIA",
}
# Published in MS-ODRAWXML but belonging to Word or Excel parts. Named so their absence is deliberate.
EXT_NOT_PPTX = {
    "word/2010/wordprocessingShape", "word/2010/wordml", "word/2010/wordprocessingGroup",
    "word/2010/wordprocessingCanvas", "word/2010/wordprocessingDrawing", "word/2012/wordprocessingDrawing",
    "excel/2010/spreadsheetDrawing",
}


def local(q):
    return (q or "").split(":")[-1]


def restriction(node):
    """The facets on a simple type, other than enumeration, as text."""
    bits = []
    for r in node.iter(f"{XS}restriction"):
        if r.get("base"):
            bits.append(f"base={local(r.get('base'))}")
        for f in r:
            tag = f.tag.replace(XS, "")
            if tag != "enumeration" and f.get("value") is not None:
                bits.append(f"{tag}={f.get('value')}")
    for u in node.iter(f"{XS}union"):
        bits.append("union=" + " ".join(local(m) for m in (u.get("memberTypes") or "").split()))
    for li in node.iter(f"{XS}list"):
        bits.append(f"list={local(li.get('itemType'))}")
    return " ".join(bits)


def occurs(n):
    lo, hi = n.get("minOccurs", "1"), n.get("maxOccurs", "1")
    return "" if (lo, hi) == ("1", "1") else f"{lo}..{hi}"


def declared(node, inherited_use=""):
    """What a type or group directly declares. Groups and bases become rows of their own,
    so a shared group is listed once and not repeated per host."""
    for ch in node:
        tag = ch.tag.replace(XS, "")
        if tag in ("sequence", "choice", "all"):
            yield from declared(ch, "choice" if tag == "choice" else inherited_use)
        elif tag in ("complexContent", "simpleContent"):
            yield from declared(ch, inherited_use)
        elif tag in ("extension", "restriction"):
            base = local(ch.get("base"))
            if base:
                yield ("base", base, base, "", "", "")
            yield from declared(ch, inherited_use)
        elif tag == "element":
            name = ch.get("name") or ch.get("ref")
            yield ("element", name, local(ch.get("type") or ch.get("ref")), occurs(ch) or inherited_use, "", "")
        elif tag == "group" and ch.get("ref"):
            yield ("group", local(ch.get("ref")), local(ch.get("ref")), occurs(ch) or inherited_use, "", "")
        elif tag == "attribute":
            name = ch.get("name") or ch.get("ref")
            use = ch.get("use", "optional")
            typ = ch.get("type") or ch.get("ref") or ""
            res = ""
            inner = ch.find(f"{XS}simpleType")
            if inner is not None:
                vals = [e.get("value") for e in inner.iter(f"{XS}enumeration")]
                res = " | ".join(vals) if vals else restriction(inner)
                typ = typ or "(inline)"
            yield ("attribute", "@" + name, local(typ), use, ch.get("default") or ch.get("fixed") or "", res)
        elif tag == "attributeGroup" and ch.get("ref"):
            yield ("attrgroup", local(ch.get("ref")), local(ch.get("ref")), "", "", "")
        elif tag == "any":
            yield ("any", "xsd:any", ch.get("namespace", ""), occurs(ch), "", ch.get("processContents", ""))
        elif tag == "anyAttribute":
            yield ("any", "@xsd:any", ch.get("namespace", ""), "", "", ch.get("processContents", ""))


def area_for(prefix, name, fixed):
    if fixed:
        return fixed
    rules = PML if prefix == "p" else DML if prefix == "a" else None
    if rules is None:
        if prefix in WHOLE_FILE:
            return WHOLE_FILE[prefix]
        raise SystemExit(f"no area rule for {prefix}:{name}")
    for rx, code in rules:
        if re.search(rx, name):
            return code
    raise SystemExit(f"no area rule for {prefix}:{name}")


def rows_for(path, prefix, fixed=None):
    root = ET.parse(path).getroot()
    rows = []

    def add(owner, kind, name, typ, use, default, res, area):
        rows.append({"id": f"{prefix}:{owner}/{name}" if owner else f"{prefix}:{name}", "area": area,
                     "owner": owner, "kind": kind, "name": name, "type": typ, "use": use,
                     "default": default, "restriction": res})

    for node in root:
        tag = node.tag.replace(XS, "")
        name = node.get("name")
        if tag == "element" and name:
            typ = local(node.get("type"))
            add("", "root", f"<{name}>", typ, "", "", "", area_for(prefix, typ or name, fixed))
            inner = node.find(f"{XS}complexType")
            if inner is not None:
                area = area_for(prefix, name, fixed)
                for kind, n, t, u, d, r in declared(inner):
                    add(name, kind, n, t, u, d, r, area)
        elif tag == "attribute" and name:
            # Global attributes: r:id and r:embed, and most of what o: adds to VML shapes.
            inner = node.find(f"{XS}simpleType")
            vals = [e.get("value") for e in inner.iter(f"{XS}enumeration")] if inner is not None else []
            res = " | ".join(vals) if vals else (restriction(inner) if inner is not None else "")
            add("", "rootattr", "@" + name, local(node.get("type")) or "(inline)", node.get("use", "optional"),
                node.get("default") or "", res, area_for(prefix, name, fixed))
        elif tag in ("complexType", "group", "attributeGroup") and name:
            area = area_for(prefix, name, fixed)
            add("", "type", name, "", "", "", "", area)
            for kind, n, t, u, d, r in declared(node):
                add(name, kind, n, t, u, d, r, area)
        elif tag == "simpleType" and name:
            area = area_for(prefix, name, fixed)
            vals = [e.get("value") for e in node.iter(f"{XS}enumeration")]
            add("", "simple", name, "", "", "", restriction(node), area)
            for v in vals:
                add(name, "value", "=" + v, "", "", "", "", area)
    seen = Counter(r["id"] for r in rows)
    # A name the schema legitimately declares twice in one type (choice branches) gets a
    # suffix so ids stay unique and stable.
    dup = Counter()
    for r in rows:
        if seen[r["id"]] > 1:
            dup[r["id"]] += 1
            if dup[r["id"]] > 1:
                r["id"] = f"{r['id']}#{dup[r['id']]}"
    return rows


def collect():
    rows, sources = [], []
    for stem, prefix in OOXML.items():
        rows += rows_for(ROOT / f"schemas/ooxml/{stem}.xsd", prefix)
        sources.append((f"schemas/ooxml/{stem}.xsd", prefix))
    for stem, prefix in OPC.items():
        rows += rows_for(ROOT / f"schemas/opc/{stem}.xsd", prefix)
        sources.append((f"schemas/opc/{stem}.xsd", prefix))
    index = json.loads((ROOT / "schemas/ms-extensions/index.json").read_text())
    skipped = []
    for fname, uri, _ in index:
        short = uri.split("schemas.microsoft.com/")[1]
        short = short.removeprefix("office/")
        if short in EXT_NOT_PPTX:
            skipped.append(short)
            continue
        if short not in EXT_AREA:
            raise SystemExit(f"no area rule for extension namespace {short}")
        rows += rows_for(ROOT / "schemas/ms-extensions" / fname, "x:" + short, EXT_AREA[short])
        sources.append((f"schemas/ms-extensions/{fname}", "x:" + short))
    return rows, sources, skipped


def read_status(path):
    if not path.exists():
        return {}
    lines = path.read_text(encoding="utf8").splitlines()
    head = lines[0].split("\t")
    return {dict(zip(head, ln.split("\t")))["id"]: dict(zip(head, ln.split("\t"))) for ln in lines[1:]}


def clean(v):
    return str(v).replace("\t", " ").replace("\n", " ").replace("|", "\\|") if v else ""


def render(rows, sources, skipped, status):
    out = {}
    for r in rows:
        kept = status.get(r["id"], {})
        for s in STATUS:
            r[s] = kept.get(s, ".")
    out["checklist.tsv"] = "\t".join(COLUMNS) + "\n" + "".join(
        "\t".join(str(r[c]).replace("\t", " ").replace("\n", " ") for c in COLUMNS) + "\n" for r in rows)

    counts = Counter(r["area"] for r in rows)
    kinds = Counter(r["kind"] for r in rows)
    idx = ["# Spec checklist", "",
           "**Generated by `tools/spec-checklist/generate.py`. Do not edit by hand**, except the",
           "status columns in `checklist.tsv`, which regeneration keeps.", "",
           "Every tag, property and enum value in the vendored schemas, each assigned to a roadmap",
           "tier. A phase cannot close while a row in its scope is unaccounted for.", "",
           "Status letters: `M` model, `I` import, `E` export, `R` render, `U` UI. Values: `.` not started,",
           "`y` done with a test, `r` retained verbatim, `n/a` does not apply. `tools/spec-audit/audit.py --check`",
           "holds every `y` to code, a test and a fixture, and `--propose` says what the evidence earns (`STAT-01`).", "",
           f"**Rows: {len(rows)}.** " + ", ".join(f"{k} {v}" for k, v in kinds.most_common()) + ".", "",
           "| Area | What | Rows | Import, export, model | Render | Edit UI | Level |", "| --- | --- | ---: | --- | --- | --- | --- |"]
    for code, (title, tier, ren, edit, level) in sorted(AREAS.items(), key=lambda kv: (kv[1][1], kv[0])):
        if counts[code]:
            idx.append(f"| [{code}]({code}.md) | {title} | {counts[code]} | Tier {tier} | {ren} | {edit} | {level} |")
    idx += ["", "## Sources", "", "| File | Namespace key |", "| --- | --- |"]
    idx += [f"| `{p}` | `{k}` |" for p, k in sources]
    idx += ["", "Excluded on purpose: these MS-ODRAWXML namespaces belong to Word and Excel parts: "
            + ", ".join(f"`{s}`" for s in sorted(skipped)) + "."]
    out["README.md"] = "\n".join(idx) + "\n"

    for code, (title, tier, ren, edit, level) in AREAS.items():
        sub = [r for r in rows if r["area"] == code]
        if not sub:
            continue
        lines = [f"# {code}: {title}", "", f"Generated. Tier {tier}; render {ren}; edit {edit}; level {level}. {len(sub)} rows.", ""]
        owner = None
        for r in sub:
            key = (r["id"].split("/")[0] if r["owner"] else r["id"])
            if r["kind"] in ("type", "simple", "root", "rootattr"):
                lines += ["", f"## `{r['id']}`" + (f" : `{r['type']}`" if r["type"] else "")]
                if r["restriction"]:
                    lines.append(f"Restriction: `{clean(r['restriction'])}`")
                owner = None
                continue
            if owner != key:
                if r["kind"] == "value":
                    pass
                else:
                    lines += ["", "| Kind | Name | Type | Use | Default | Restriction |", "| --- | --- | --- | --- | --- | --- |"]
                owner = key
            if r["kind"] == "value":
                lines.append(f"- `{r['name'][1:]}`")
            else:
                lines.append(f"| {r['kind']} | `{clean(r['name'])}` | `{clean(r['type'])}` | {clean(r['use'])} | {clean(r['default'])} | {clean(r['restriction'])} |")
        out[f"{code}.md"] = "\n".join(lines) + "\n"
    return out


def main():
    rows, sources, skipped = collect()
    ids = Counter(r["id"] for r in rows)
    repeated = [k for k, v in ids.items() if v > 1]
    if repeated:
        raise SystemExit(f"duplicate row ids: {repeated[:5]}")
    files = render(rows, sources, skipped, read_status(OUT / "checklist.tsv"))
    if "--check" in sys.argv:
        stale = [n for n, t in files.items() if not (OUT / n).exists() or (OUT / n).read_text(encoding="utf8") != t]
        if stale:
            raise SystemExit(f"docs/spec is stale: {stale[:5]}. Run tools/spec-checklist/generate.py")
        print(f"spec checklist current: {len(rows)} rows")
        return
    OUT.mkdir(parents=True, exist_ok=True)
    for n, t in files.items():
        (OUT / n).write_text(t, encoding="utf8")
    print(f"{len(rows)} rows, {len(files)} files")
    print(Counter(r["kind"] for r in rows).most_common())
    print(sorted(Counter(r["area"] for r in rows).items()))


if __name__ == "__main__":
    main()
