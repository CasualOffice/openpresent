"""Decks for the checklist rows of Tiers 1A to 1D that nothing else exercised (CLOSE-02).

A row with no fixture cannot be marked, so these exist for the audit's walk and for the corpus
tests every deck goes through. They are schema-valid, and they say nothing about what PowerPoint
does with them. Imported by generate.py, which owns the helpers; separate so that sessions adding
fixtures elsewhere do not collide here.
"""
import csv
import pathlib

from generate import (CORE_NS, CP_NS, DECL, EP_NS, FMTID, NS, PKG_RELS, PML, SCHEME, T, THEME, TREE, deck, full_deck,
                      pres_doc, rels, replace, xf, zipped)

ROOT = pathlib.Path(__file__).resolve().parents[2]
A14 = "http://schemas.microsoft.com/office/drawing/2010/main"
A15 = "http://schemas.microsoft.com/office/drawing/2012/main"
A16 = "http://schemas.microsoft.com/office/drawing/2014/main"
A1611 = "http://schemas.microsoft.com/office/drawing/2016/11/main"
AHYP = "http://schemas.microsoft.com/office/drawing/2018/hyperlinkcolor"
P2023 = "http://schemas.microsoft.com/office/powerpoint/2023/02/main"
THM15 = "http://schemas.microsoft.com/office/thememl/2012/main"
P_NS = "http://schemas.openxmlformats.org/presentationml/2006/main"
A_NS = "http://schemas.openxmlformats.org/drawingml/2006/main"
MC_NS = "http://schemas.openxmlformats.org/markup-compatibility/2006"
LC = "http://schemas.openxmlformats.org/drawingml/2006/lockedCanvas"
DS = "http://www.w3.org/2000/09/xmldsig#"
MDSSI = "http://schemas.openxmlformats.org/package/2006/digital-signature"
GUID = "{00000000-0000-0000-0000-000000000001}"
GUID2 = "{00000000-0000-0000-0000-000000000002}"
EXT = '<p:extLst><p:ext uri="{C}"/></p:extLst>'
A_EXT = '<a:extLst><a:ext uri="{C}"/></a:extLst>'
XFRM = xf(1, 2, 3, 4, ' rot="60000" flipH="1" flipV="1"')


def values(prefix, simple):
    """A simple type's enumeration, from the checklist, in schema order."""
    with open(ROOT / "docs/spec/checklist.tsv", encoding="utf8", newline="") as handle:
        rows = csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE)
        return [r["name"][1:] for r in rows if r["kind"] == "value" and r["owner"] == simple and r["id"].startswith(prefix + ":")]


# ---------------------------------------------------------------- package

def variant_value(typ):
    return {"i1": "-1", "i2": "-2", "i4": "-4", "i8": "-8", "int": "7", "ui1": "1", "ui2": "2", "ui4": "4", "ui8": "8", "uint": "9", "r4": "1.5",
            "r8": "2.5", "decimal": "1.25", "lpstr": "a", "lpwstr": "b", "bstr": "c", "date": "2020-01-02T03:04:05Z", "filetime": "2020-01-02T03:04:05Z",
            "bool": "true", "cy": "1.2500", "error": "0x80000000", "clsid": FMTID}[typ]


def package_every_variant():
    vectors = [t for t in values("vt", "ST_VectorBaseType") if t != "variant"]
    arrays = [t for t in values("vt", "ST_ArrayBaseType") if t != "variant"]
    plain = ["i1", "i2", "i4", "i8", "int", "ui1", "ui2", "ui4", "ui8", "uint", "r4", "r8", "decimal", "lpstr", "lpwstr", "bstr", "date",
             "filetime", "bool", "cy", "error", "clsid"]
    inner = [f"<vt:{t}>{variant_value(t)}</vt:{t}>" for t in plain]
    inner += ["<vt:empty/>", "<vt:null/>", "<vt:blob>QUJD</vt:blob>", "<vt:oblob>QUJD</vt:oblob>", "<vt:stream>QUJD</vt:stream>",
              "<vt:ostream>QUJD</vt:ostream>", "<vt:storage>QUJD</vt:storage>", "<vt:ostorage>QUJD</vt:ostorage>",
              f'<vt:vstream version="{FMTID}">QUJD</vt:vstream>', "<vt:variant><vt:i4>1</vt:i4></vt:variant>",
              '<vt:vector size="1" baseType="i4"><vt:i4>1</vt:i4></vt:vector>',
              '<vt:array lBounds="0" uBounds="0" baseType="i4"><vt:i4>1</vt:i4></vt:array>']
    pid = 2
    props = []

    def prop(body):
        nonlocal pid
        props.append(f'<property fmtid="{FMTID}" pid="{pid}" name="p{pid}">{body}</property>')
        pid += 1

    prop(f'<vt:vector size="{len(inner)}" baseType="variant">' + "".join(f"<vt:variant>{v}</vt:variant>" for v in inner) + "</vt:vector>")
    for t in vectors:
        prop(f'<vt:vector size="1" baseType="{t}"><vt:{t}>{variant_value(t)}</vt:{t}></vt:vector>')
    for t in arrays:
        prop(f'<vt:array lBounds="0" uBounds="0" baseType="{t}"><vt:{t}>{variant_value(t)}</vt:{t}></vt:array>')
    prop('<vt:array lBounds="0" uBounds="0" baseType="variant"><vt:variant><vt:i4>1</vt:i4></vt:variant></vt:array>')
    custom = f'{DECL}<Properties {CP_NS}>' + "".join(props) + "</Properties>"
    core = (f'{DECL}<cp:coreProperties {CORE_NS}><cp:keywords xml:lang="en"><cp:value xml:lang="de">eins</cp:value></cp:keywords>'
            "</cp:coreProperties>")
    app = f'{DECL}<Properties {EP_NS}><DigSig><vt:blob>QUJD</vt:blob></DigSig></Properties>'
    entries = full_deck(pres_doc(), core=core, app=app, custom=custom)
    # An explicit Internal target mode, which is also the default, and an External one.
    root = rels(("officeDocument", "ppt/presentation.xml", "Internal"), (f"{PKG_RELS}/metadata/core-properties", "docProps/core.xml"),
                ("extended-properties", "docProps/app.xml"), ("custom-properties", "docProps/custom.xml"),
                ("hyperlink", "https://example.com/", "External"))
    return replace(entries, "_rels/.rels", root)


def signed_deck():
    """A digital signature, retained and never checked here (`CS-FND-0009`)."""
    signature = (f'{DECL}<Signature xmlns="{DS}"><SignedInfo><CanonicalizationMethod Algorithm="urn:c"/><SignatureMethod Algorithm="urn:s"/>'
                 '<Reference URI="#idPackageObject"><DigestMethod Algorithm="urn:d"/><DigestValue>QUJD</DigestValue></Reference></SignedInfo>'
                 '<SignatureValue>QUJD</SignatureValue><Object Id="idPackageObject"><Manifest><Reference URI="/ppt/_rels/presentation.xml.rels?ContentType=x">'
                 '<Transforms><Transform Algorithm="http://schemas.openxmlformats.org/package/2006/RelationshipTransform">'
                 f'<mdssi:RelationshipReference xmlns:mdssi="{MDSSI}" SourceId="rId1"/>'
                 f'<mdssi:RelationshipsGroupReference xmlns:mdssi="{MDSSI}" SourceType="{T}/slide"/></Transform></Transforms>'
                 '<DigestMethod Algorithm="urn:d"/><DigestValue>QUJD</DigestValue></Reference></Manifest>'
                 '<SignatureProperties><SignatureProperty Id="idSignatureTime" Target="#idPackageSignature">'
                 f'<mdssi:SignatureTime xmlns:mdssi="{MDSSI}"><mdssi:Format>YYYY-MM-DDThh:mm:ssTZD</mdssi:Format>'
                 '<mdssi:Value>2020-01-02T03:04:05Z</mdssi:Value></mdssi:SignatureTime></SignatureProperty></SignatureProperties></Object></Signature>')
    origin = f"{PKG_RELS}/digital-signature/origin"
    types = [("/_xmlsignatures/origin.sigs", "application/vnd.openxmlformats-package.digital-signature-origin"),
             ("/_xmlsignatures/sig1.xml", "application/vnd.openxmlformats-package.digital-signature-xmlsignature+xml")]
    parts = [("_xmlsignatures/origin.sigs", b""), ("_xmlsignatures/sig1.xml", signature),
             ("_xmlsignatures/_rels/origin.sigs.rels", rels((f"{PKG_RELS}/digital-signature/signature", "sig1.xml")))]
    return deck(types_extra=types, root_extra=[(origin, "_xmlsignatures/origin.sigs")], parts=parts)


def mce_preserve_deck():
    """`mc:PreserveElements` and `mc:PreserveAttributes`, which the resolver accepts and bytes keep."""
    x = "urn:fixture:preserve"
    slide = (f'{DECL}<p:sld {NS} xmlns:mc="{MC_NS}" xmlns:x="{x}" mc:Ignorable="x" mc:PreserveElements="x:keep" mc:PreserveAttributes="x:mark" '
             f'x:mark="1"><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld></p:sld>')
    return replace(deck(), "ppt/slides/slide1.xml", slide)


# ---------------------------------------------------------------- presentation and view: one value of each per deck

def nth(prefix, simple, i):
    found = values(prefix, simple)
    return found[i % len(found)]


def pres_values_deck(i):
    """Deck `i` of a set: each attribute takes its `i`th value, as one part can state only one."""
    size = nth("p", "ST_SlideSizeType", i)
    conformance = nth("st", "ST_ConformanceClass", i)
    show_type = ['<p:present/>', '<p:browse showScrollbar="0"/>', '<p:kiosk restart="300000"/>'][i % 3]
    slides = ['<p:sldAll/>', '<p:sldRg st="1" end="2"/>', '<p:custShow id="0"/>'][i % 3]
    verifier = (f'<p:modifyVerifier algorithmName="SHA-512" hashValue="QUJD" saltValue="QUJD" spinValue="100000" '
                f'cryptProviderType="{nth("st", "ST_CryptProv", i)}" cryptAlgorithmClass="{nth("st", "ST_AlgClass", i)}" '
                f'cryptAlgorithmType="{nth("st", "ST_AlgType", i)}" cryptAlgorithmSid="4" spinCount="100000" saltData="QUJD" hashData="QUJD" '
                'cryptProvider="p" algIdExt="1" algIdExtSource="s" cryptProviderTypeExt="1" cryptProviderTypeExtSource="s"/>')
    body = ('<p:embeddedFontLst><p:embeddedFont><p:font typeface="F"/><p:regular r:id="rId9"/><p:bold r:id="rId9"/><p:italic r:id="rId9"/>'
            '<p:boldItalic r:id="rId9"/></p:embeddedFont></p:embeddedFontLst>'
            f'<p:custShowLst><p:custShow name="S" id="0"><p:sldLst><p:sld r:id="rId3"/></p:sldLst>{EXT}</p:custShow></p:custShowLst>'
            f'<p:photoAlbum bw="1" showCaptions="1" layout="{nth("p", "ST_PhotoAlbumLayout", i)}" frame="{nth("p", "ST_PhotoAlbumFrameShape", i)}">{EXT}</p:photoAlbum>'
            '<p:custDataLst><p:tags r:id="rId8"/></p:custDataLst>'
            '<p:kinsoku lang="ja-JP" invalStChars="a" invalEndChars="b"/>'
            f'{verifier}')
    pres = pres_doc(body, f' serverZoom="50000" firstSlideNum="2" conformance="{conformance}" compatMode="1" showSpecialPlsOnTitleSld="0" removePersonalInfoOnSave="1" '
                          'strictFirstAndLastChars="0" saveSubsetFonts="1" autoCompressPictures="0"')
    pres = pres.replace('<p:sldSz cx="12192000" cy="6858000"/>', f'<p:sldSz cx="12192000" cy="6858000" type="{size}"/>')
    pres = pres.replace('</p:sldMasterIdLst>', '</p:sldMasterIdLst><p:notesMasterIdLst><p:notesMasterId r:id="rId11"/></p:notesMasterIdLst>'
                        '<p:handoutMasterIdLst><p:handoutMasterId r:id="rId12"/></p:handoutMasterIdLst>')
    pres_props = (f'{DECL}<p:presentationPr {NS}><p:htmlPubPr showSpeakerNotes="1" r:id="rId1">{slides}{EXT}</p:htmlPubPr>'
                  f'<p:webPr showAnimation="1" resizeGraphics="0" allowPng="1" relyOnVml="1" organizeInFolders="0" useLongFilenames="0" '
                  f'imgSz="{nth("p", "ST_WebScreenSize", i)}" encoding="utf-8" clr="{nth("p", "ST_WebColorType", i)}">{EXT}</p:webPr>'
                  f'<p:prnPr prnWhat="{nth("p", "ST_PrintWhat", i)}" clrMode="{nth("p", "ST_PrintColorMode", i)}" hiddenSlides="1" '
                  f'scaleToFitPaper="1" frameSlides="1">{EXT}</p:prnPr>'
                  f'<p:showPr loop="1" showNarration="1" showAnimation="0" useTimings="0">{show_type}{slides}</p:showPr></p:presentationPr>')
    view_props = (f'{DECL}<p:viewPr {NS} lastView="{nth("p", "ST_ViewType", i)}" showComments="1">'
                  f'<p:normalViewPr vertBarState="{nth("p", "ST_SplitterBarState", i)}" horzBarState="{nth("p", "ST_SplitterBarState", i + 1)}">'
                  f'<p:restoredLeft sz="1"/><p:restoredTop sz="2"/>{EXT}</p:normalViewPr></p:viewPr>')
    if i == 0:  # every holder's own extension list, once
        common = '<p:cViewPr><p:scale><a:sx n="1" d="1"/><a:sy n="1" d="1"/></p:scale><p:origin x="0" y="0"/></p:cViewPr>'
        slide_view = f'<p:cSldViewPr snapToGrid="0" snapToObjects="1" showGuides="1">{common}<p:guideLst/></p:cSldViewPr>'
        view_props = view_props.replace('</p:viewPr>',
                                        f'<p:slideViewPr>{slide_view}{EXT}</p:slideViewPr><p:outlineViewPr>{common}{EXT}</p:outlineViewPr>'
                                        f'<p:notesTextViewPr>{common}{EXT}</p:notesTextViewPr><p:sorterViewPr>{common}{EXT}</p:sorterViewPr>'
                                        f'<p:notesViewPr>{slide_view}{EXT}</p:notesViewPr>{EXT}</p:viewPr>')
    entries = full_deck(pres, pres_props=pres_props, view_props=view_props)
    return entries


# ---------------------------------------------------------------- slide structure

def ext(uri, body):
    return f'<a:ext uri="{uri}">{body}</a:ext>'


def nv_ext(body):
    return f'<a:extLst>{body}</a:extLst>'


def shape_every_construct():
    pitch = "".join(f'<a:r><a:rPr lang="en-US"><a:latin typeface="F" pitchFamily="{v}"/></a:rPr><a:t>p</a:t></a:r>' for v in values("a", "ST_PitchFamily"))
    screen = "".join(ext("{B}", f'<a15:backgroundPr xmlns:a15="{A15}" bwMode="auto" bwPure="1" bwNormal="1" targetScreenSize="{v}"/>')
                     for v in values("x:drawing/2012/main", "ST_TargetScreenSz"))
    signature = (f'<a15:signatureLine xmlns:a15="{A15}" isSignatureLine="1" id="{GUID}" provId="{GUID2}" signingInstructionsSet="1" allowComments="1" '
                 'showSignDate="1" suggestedSigner="a" suggestedSigner2="b" suggestedSignerEmail="c" signingInstructions="d" addlXml="e" sigProvUrl="f"/>')
    link_colors = "".join(ext("{C}", f'<ahyp:hlinkClr xmlns:ahyp="{AHYP}" val="{v}"/>') for v in values("x:drawing/2018/hyperlinkcolor", "ST_HyperlinkColor"))
    link = (f'<a:hlinkClick r:id="rId2" invalidUrl="u" action="ppaction://noaction" tgtFrame="_blank" tooltip="t" history="0" highlightClick="1" endSnd="1">'
            f'<a:snd r:embed="rId3" name="s"/>{nv_ext(link_colors)}</a:hlinkClick>')
    cnv_ext = nv_ext(ext("{A}", f'<a16:predDERef xmlns:a16="{A16}" pred="{GUID}"/>') + ext("{B}", f'<a16:creationId xmlns:a16="{A16}" id="{GUID2}"/>')
                     + ext("{C}", f'<a1611:picAttrSrcUrl xmlns:a1611="{A1611}" r:id="rId2"/>'))
    sp = (f'<p:sp useBgFill="1"><p:nvSpPr><p:cNvPr id="2" name="Shape" descr="d" hidden="0" title="t">{link}{cnv_ext}</p:cNvPr>'
          f'<p:cNvSpPr txBox="1"><a:spLocks noGrp="1" noSelect="1" noRot="1" noChangeAspect="1" noMove="1" noResize="1" noEditPoints="1" noAdjustHandles="1" '
          f'noChangeArrowheads="1" noChangeShapeType="1" noTextEdit="1">{A_EXT}</a:spLocks>{nv_ext(ext("{B}", signature) + screen)}</p:cNvSpPr>'
          f'<p:nvPr isPhoto="1" userDrawn="1"><a:wavAudioFile r:embed="rId3"/><p:custDataLst><p:tags r:id="rId4"/></p:custDataLst>{EXT}</p:nvPr></p:nvSpPr>'
          f'<p:spPr bwMode="auto">{XFRM}<a:prstGeom prst="rect"><a:avLst/></a:prstGeom>{A_EXT}</p:spPr>'
          f'<p:txBody><a:bodyPr/><a:lstStyle/><a:p>{pitch}</a:p></p:txBody>{EXT}</p:sp>')
    style = ('<p:style><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef>'
             '<a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="lt1"/></a:fontRef></p:style>')
    cxn_ext = nv_ext(ext("{A}", f'<a16:cxnDERefs xmlns:a16="{A16}" st="{GUID}" end="{GUID2}"/>'))
    cxn = (f'<p:cxnSp><p:nvCxnSpPr><p:cNvPr id="3" name="Connector">{cxn_ext}</p:cNvPr><p:cNvCxnSpPr><a:cxnSpLocks noMove="1" noEditPoints="1">{A_EXT}</a:cxnSpLocks>'
           f'<a:stCxn id="2" idx="0"/><a:endCxn id="2" idx="1"/>{A_EXT}</p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/>{style}{EXT}</p:cxnSp>')
    pic = ('<p:pic><p:nvPicPr><p:cNvPr id="4" name="Picture"/><p:cNvPicPr preferRelativeResize="0"><a:picLocks noCrop="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr>'
           f'<p:blipFill><a:blip r:embed="rId5"/><a:stretch/></p:blipFill><p:spPr/>{style}{EXT}</p:pic>')
    gsp = '<a:sp><a:nvSpPr><a:cNvPr id="6" name="c"/><a:cNvSpPr/></a:nvSpPr><a:spPr/><a:txSp><a:txBody><a:bodyPr/><a:p/></a:txBody><a:useSpRect/></a:txSp>{s}{e}</a:sp>'
    astyle = style.replace("p:style", "a:style")
    canvas = (f'<lc:lockedCanvas xmlns:lc="{LC}"><a:nvGrpSpPr><a:cNvPr id="5" name="canvas"/><a:cNvGrpSpPr/></a:nvGrpSpPr><a:grpSpPr/>'
              '<a:txSp><a:txBody><a:bodyPr/><a:p/></a:txBody><a:useSpRect/></a:txSp>'
              + gsp.format(s=astyle, e=A_EXT) +
              f'<a:cxnSp><a:nvCxnSpPr><a:cNvPr id="7" name="c"/><a:cNvCxnSpPr/></a:nvCxnSpPr><a:spPr/>{astyle}{A_EXT}</a:cxnSp>'
              f'<a:pic><a:nvPicPr><a:cNvPr id="8" name="c"/><a:cNvPicPr/></a:nvPicPr><a:blipFill/><a:spPr/></a:pic>'
              f'<a:graphicFrame><a:nvGraphicFramePr><a:cNvPr id="9" name="c"/><a:cNvGraphicFramePr/></a:nvGraphicFramePr>'
              f'<a:graphic><a:graphicData uri="urn:g"/></a:graphic><a:xfrm><a:off x="0" y="0"/><a:ext cx="1" cy="1"/></a:xfrm>{A_EXT}</a:graphicFrame>'
              f'<a:grpSp><a:nvGrpSpPr><a:cNvPr id="10" name="c"/><a:cNvGrpSpPr/></a:nvGrpSpPr><a:grpSpPr/>{A_EXT}</a:grpSp>'
              f'{A_EXT}</lc:lockedCanvas>')
    obj = nv_ext(ext("{A}", f'<a15:objectPr xmlns:a15="{A15}" objectId="o" isActiveX="1" linkType="l"/>'))
    frame = (f'<p:graphicFrame bwMode="auto"><p:nvGraphicFramePr><p:cNvPr id="11" name="Frame">{obj}</p:cNvPr><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1" noDrilldown="1" '
             f'noSelect="1" noChangeAspect="1" noMove="1" noResize="1">{A_EXT}</a:graphicFrameLocks>{A_EXT}</p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>'
             f'<p:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></p:xfrm><a:graphic><a:graphicData uri="{LC}">{canvas}</a:graphicData></a:graphic>{EXT}</p:graphicFrame>')
    p14 = "http://schemas.microsoft.com/office/powerpoint/2010/main"
    content_part = (f'<mc:AlternateContent xmlns:mc="{MC_NS}"><mc:Choice xmlns:p14="{p14}" Requires="p14"><p14:contentPart bwMode="auto" r:id="rId2"><p14:nvContentPartPr>'
                    f'<p14:cNvPr id="12" name="Part"/><p14:cNvContentPartPr isComment="0"><a:cpLocks noMove="1">{A_EXT}</a:cpLocks>{A_EXT}</p14:cNvContentPartPr><p14:nvPr/>'
                    '</p14:nvContentPartPr></p14:contentPart></mc:Choice><mc:Fallback/></mc:AlternateContent><p:contentPart r:id="rId2"/>')
    legacy = nv_ext(ext("{A}", f'<a15:nonVisualGroupProps xmlns:a15="{A15}" isLegacyGroup="1"/>'))
    group = (f'<p:grpSp><p:nvGrpSpPr><p:cNvPr id="13" name="Group">{A_EXT}</p:cNvPr><p:cNvGrpSpPr><a:grpSpLocks noGrp="1" noUngrp="1" noSelect="1" noRot="1" noChangeAspect="1" noMove="1" '
             f'noResize="1">{A_EXT}</a:grpSpLocks>{legacy}</p:cNvGrpSpPr><p:nvPr/></p:nvGrpSpPr><p:grpSpPr bwMode="auto"><a:xfrm rot="60000" flipH="1" flipV="1"><a:off x="1" y="2"/>'
             f'<a:ext cx="3" cy="4"/><a:chOff x="1" y="2"/><a:chExt cx="3" cy="4"/></a:xfrm><a:noFill/>{A_EXT}</p:grpSpPr>{EXT}</p:grpSp>')
    extra_part = (f'{DECL}<p:sldSyncPr {NS} serverSldId="s" serverSldModifiedTime="2020-01-02T03:04:05Z" clientInsertedTime="2020-01-02T03:04:05Z">{EXT}</p:sldSyncPr>')
    bg_slide = f'<p:bg bwMode="auto"><p:bgPr shadeToTitle="1"><a:noFill/><a:effectLst/>{EXT}</p:bgPr></p:bg>'
    slide = (f'{DECL}<p:sld {NS} showMasterSp="0" showMasterPhAnim="0" show="0"><p:cSld name="n">{bg_slide}<p:spTree>{TREE}{sp}{cxn}{pic}{frame}{content_part}{group}</p:spTree>'
             f'<p:custDataLst><p:tags r:id="rId4"/></p:custDataLst><p:controls><p:control spid="2" name="c" r:id="rId4" showAsIcon="1" imgW="1" imgH="1"/></p:controls>{EXT}</p:cSld>'
             f'<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{EXT}</p:sld>')
    hf = f'<p:hf sldNum="0" hdr="0" ftr="0" dt="0">{EXT}</p:hf>'
    ph_type = f'<p:ph type="body" orient="vert" sz="half" idx="1" hasCustomPrompt="1">{EXT}</p:ph>'
    ph_ext = f'<p:extLst><p:ext uri="{{E}}"><p2023:phTypeExt xmlns:p2023="{P2023}"><p2023:type><p2023:cameo/></p2023:type></p:ext></p:extLst>'
    ph_ext2 = f'<p:extLst><p:ext uri="{{E}}"><p2023:phTypeExt xmlns:p2023="{P2023}"><p2023:type><p2023:unknown/></p2023:type></p2023:phTypeExt></p:ext></p:extLst>'
    ph_ext = ph_ext.replace("</p2023:type></p:ext>", "</p2023:type></p2023:phTypeExt></p:ext>")
    timing = '<p:timing><p:tnLst><p:par><p:cTn id="1"/></p:par></p:tnLst></p:timing>'
    transition = '<p:transition/>'
    styles = (f'<p:txStyles><p:titleStyle/><p:bodyStyle/><p:otherStyle/>{EXT}</p:txStyles>')
    layout = (f'{DECL}<p:sldLayout {NS} showMasterSp="0" showMasterPhAnim="0" matchingName="m" type="titleOnly" preserve="1" userDrawn="1"><p:cSld name="L"><p:spTree>{TREE}'
              f'<p:sp><p:nvSpPr><p:cNvPr id="2" name="p"/><p:cNvSpPr/><p:nvPr>{ph_type}{ph_ext}</p:nvPr></p:nvSpPr><p:spPr/></p:sp>'
              f'<p:sp><p:nvSpPr><p:cNvPr id="3" name="q"/><p:cNvSpPr/><p:nvPr><p:ph idx="2"/>{ph_ext2}</p:nvPr></p:nvSpPr><p:spPr/></p:sp></p:spTree></p:cSld>'
              f'<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{transition}{timing}{hf}<p:extLst mod="1"><p:ext uri="{{C}}"/></p:extLst></p:sldLayout>')
    clrmap = ('<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" '
              f'accent6="accent6" hlink="hlink" folHlink="folHlink">{A_EXT}</p:clrMap>')
    master = (f'{DECL}<p:sldMaster {NS} preserve="1"><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>{clrmap}'
              f'<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1">{EXT}</p:sldLayoutId></p:sldLayoutIdLst>{transition}{timing}{hf}{styles}'
              f'<p:extLst mod="1"><p:ext uri="{{C}}"/></p:extLst></p:sldMaster>')
    notes_master = (f'{DECL}<p:notesMaster {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>{clrmap}{hf}<p:notesStyle/>'
                    f'<p:extLst mod="1"><p:ext uri="{{C}}"/></p:extLst></p:notesMaster>')
    handout = (f'{DECL}<p:handoutMaster {NS}><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>{clrmap}{hf}'
               f'<p:extLst mod="1"><p:ext uri="{{C}}"/></p:extLst></p:handoutMaster>')
    notes = (f'{DECL}<p:notes {NS} showMasterSp="0" showMasterPhAnim="0"><p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>'
             f'<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr><p:extLst mod="1"><p:ext uri="{{C}}"/></p:extLst></p:notes>')
    pres = (f'{DECL}<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1">{EXT}</p:sldMasterId></p:sldMasterIdLst>'
            f'<p:notesMasterIdLst><p:notesMasterId r:id="rId5">{EXT}</p:notesMasterId></p:notesMasterIdLst>'
            f'<p:handoutMasterIdLst><p:handoutMasterId r:id="rId6">{EXT}</p:handoutMasterId></p:handoutMasterIdLst>'
            f'<p:sldIdLst><p:sldId id="256" r:id="rId3">{EXT}</p:sldId></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/>'
            f'</p:presentation>')
    types = [("/ppt/notesMasters/notesMaster1.xml", f"{PML}.notesMaster+xml"), ("/ppt/handoutMasters/handoutMaster1.xml", f"{PML}.handoutMaster+xml"),
             ("/ppt/notesSlides/notesSlide1.xml", f"{PML}.notesSlide+xml"), ("/ppt/slideUpdateInfo/slideUpdateInfo1.xml", f"{PML}.slideUpdateInfo+xml"),
             ("/ppt/tags/tag1.xml", f"{PML}.tags+xml"), ("/ppt/media/audio1.wav", "audio/wav"), ("/ppt/media/image1.png", "image/png"),
             ("/ppt/theme/theme2.xml", "application/vnd.openxmlformats-officedocument.theme+xml"), ("/ppt/theme/theme3.xml", "application/vnd.openxmlformats-officedocument.theme+xml")]
    parts = [("ppt/notesMasters/notesMaster1.xml", notes_master), ("ppt/notesMasters/_rels/notesMaster1.xml.rels", rels(("theme", "../theme/theme2.xml"))),
             ("ppt/handoutMasters/handoutMaster1.xml", handout), ("ppt/handoutMasters/_rels/handoutMaster1.xml.rels", rels(("theme", "../theme/theme3.xml"))),
             ("ppt/theme/theme2.xml", THEME), ("ppt/theme/theme3.xml", THEME),
             ("ppt/notesSlides/notesSlide1.xml", notes), ("ppt/notesSlides/_rels/notesSlide1.xml.rels", rels(("notesMaster", "../notesMasters/notesMaster1.xml"), ("slide", "../slides/slide1.xml"))),
             ("ppt/slideUpdateInfo/slideUpdateInfo1.xml", extra_part), ("ppt/tags/tag1.xml", f'{DECL}<p:tagLst {NS}><p:tag name="K" val="V"/></p:tagLst>'),
             ("ppt/media/audio1.wav", b"RIFF"), ("ppt/media/image1.png", b"\x89PNG")]
    entries = deck(presentation=pres, types_extra=types, parts=parts)
    entries = replace(entries, "ppt/slides/slide1.xml", slide)
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(
        ("slideLayout", "../slideLayouts/slideLayout1.xml"), ("hyperlink", "https://example.com/", "External"), ("audio", "../media/audio1.wav"),
        ("tags", "../tags/tag1.xml"), ("image", "../media/image1.png"), ("notesSlide", "../notesSlides/notesSlide1.xml"),
        ("slideUpdateInfo", "../slideUpdateInfo/slideUpdateInfo1.xml")))
    entries = replace(entries, "ppt/slideLayouts/slideLayout1.xml", layout)
    entries = replace(entries, "ppt/slideMasters/slideMaster1.xml", master)
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"), ("presProps", "presProps.xml"),
        ("notesMaster", "notesMasters/notesMaster1.xml"), ("handoutMaster", "handoutMasters/handoutMaster1.xml")))
    # rId4 of the presentation is its properties part, so it must exist.
    entries.append(("ppt/presProps.xml", f"{DECL}<p:presentationPr {NS}/>"))
    entries = replace(entries, "[Content_Types].xml", dict(entries)["[Content_Types].xml"].replace(
        "</Types>", f'<Override PartName="/ppt/presProps.xml" ContentType="{PML}.presProps+xml"/></Types>'))
    return entries


# ---------------------------------------------------------------- theme

# A transform under each kind of colour: `EG_ColorTransform` in each owner.
TRANSFORMED = "".join(f'<a:custClr name="t{i}"><a:{c}><a:alpha val="50000"/></a:{c.split()[0]}></a:custClr>' for i, c in enumerate((
    'scrgbClr r="1000" g="2000" b="3000"', 'srgbClr val="010203"', 'hslClr hue="60000" sat="1000" lum="2000"',
    'sysClr val="window" lastClr="FFFFFF"', 'schemeClr val="accent1"', 'prstClr val="red"')))


def theme_every_construct():
    fmt = THEME[THEME.index("<a:fmtScheme"):THEME.index("</a:themeElements>")]
    clr = f'<a:clrScheme name="C">{SCHEME}{A_EXT}</a:clrScheme>'
    fonts = (f'<a:fontScheme name="F"><a:majorFont><a:latin typeface="A"/><a:ea typeface="B"/><a:cs typeface="C"/><a:font script="Jpan" typeface="D"/>{A_EXT}</a:majorFont>'
             f'<a:minorFont><a:latin typeface="A"/><a:ea typeface="B"/><a:cs typeface="C"/><a:font script="Arab" typeface="E"/></a:minorFont>{A_EXT}</a:fontScheme>')
    clrmap = ('<a:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" '
              f'accent6="accent6" hlink="hlink" folHlink="folHlink">{A_EXT}</a:clrMap>')
    style = ('<a:style><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef>'
             '<a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="{font}"><a:schemeClr val="lt1"/></a:fontRef></a:style>')
    defaults = "<a:objectDefaults>" + "".join(
        f'<a:{t}><a:spPr/><a:bodyPr/><a:lstStyle/>{style.format(font=f)}{A_EXT}</a:{t}>' for t, f in (("spDef", "major"), ("lnDef", "minor"), ("txDef", "none"))
    ) + A_EXT + "</a:objectDefaults>"
    family = ext("{05A4C25C-085E-4340-85A3-A5531E510DB2}", f'<thm15:themeFamily xmlns:thm15="{THM15}" name="N" id="{GUID}" vid="{GUID2}"><thm15:extLst><a:ext uri="{{C}}"/></thm15:extLst></thm15:themeFamily>')
    theme = (f'{DECL}<a:theme xmlns:a="{A_NS}" xmlns:r="{T}" name="T"><a:themeElements>{clr}{fonts}{fmt}{A_EXT}</a:themeElements>{defaults}'
             f'<a:extraClrSchemeLst><a:extraClrScheme>{clr}{clrmap}</a:extraClrScheme></a:extraClrSchemeLst>'
             f'<a:custClrLst><a:custClr name="c"><a:srgbClr val="010203"/></a:custClr>{TRANSFORMED}</a:custClrLst><a:extLst>{family}</a:extLst></a:theme>')
    override = f'{DECL}<a:themeOverride xmlns:a="{A_NS}">{clr}{fonts}{fmt}</a:themeOverride>'
    manager = f'{DECL}<a:themeManager xmlns:a="{A_NS}"/>'
    on_off = values("a", "ST_OnOffStyleType")
    tx = "".join(f'<a:{part}><a:tcTxStyle b="{v}" i="{v}"><a:fontRef idx="minor"><a:schemeClr val="dk1"/></a:fontRef></a:tcTxStyle><a:tcStyle><a:tcBdr/>{{fill}}</a:tcStyle></a:{part}>'
                 for part, v in zip(("wholeTbl", "band1H", "band2H"), on_off))
    tx = tx.format(fill='<a:fill><a:noFill/></a:fill>') + (
        '<a:band1V><a:tcTxStyle><a:font><a:latin typeface="A"/><a:ea typeface="B"/><a:cs typeface="C"/></a:font></a:tcTxStyle>'
        '<a:tcStyle><a:tcBdr/><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef></a:tcStyle></a:band1V>')
    refs = '<a:tblBg><a:fillRef idx="1"><a:schemeClr val="accent1"/></a:fillRef><a:effectRef idx="1"><a:schemeClr val="accent1"/></a:effectRef></a:tblBg>'
    direct = '<a:tblBg><a:fill><a:noFill/></a:fill><a:effect><a:effectLst/></a:effect></a:tblBg>'
    tables = (f'{DECL}<a:tblStyleLst xmlns:a="{A_NS}" def="{GUID}"><a:tblStyle styleId="{GUID}" styleName="S">{refs}{tx}</a:tblStyle>'
              f'<a:tblStyle styleId="{GUID2}" styleName="D">{direct}</a:tblStyle></a:tblStyleLst>')
    types = [("/ppt/theme/themeOverride1.xml", "application/vnd.openxmlformats-officedocument.themeOverride+xml"),
             ("/ppt/theme/themeManager.xml", "application/vnd.openxmlformats-officedocument.themeManager+xml"),
             ("/ppt/tableStyles.xml", f"{PML}.tableStyles+xml")]
    parts = [("ppt/theme/themeOverride1.xml", override), ("ppt/theme/themeManager.xml", manager), ("ppt/tableStyles.xml", tables)]
    entries = deck(types_extra=types, parts=parts)
    entries = replace(entries, "ppt/theme/theme1.xml", theme)
    entries = replace(entries, "ppt/slides/_rels/slide1.xml.rels", rels(("slideLayout", "../slideLayouts/slideLayout1.xml"), ("themeOverride", "../theme/themeOverride1.xml")))
    slide = dict(entries)["ppt/slides/slide1.xml"].replace("</p:cSld></p:sld>", "</p:cSld><p:clrMapOvr><a:overrideClrMapping bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/></p:clrMapOvr></p:sld>")
    entries = replace(entries, "ppt/slides/slide1.xml", slide)
    entries = replace(entries, "ppt/_rels/presentation.xml.rels", rels(
        ("slideMaster", "slideMasters/slideMaster1.xml"), ("theme", "theme/theme1.xml"), ("slide", "slides/slide1.xml"), ("tableStyles", "tableStyles.xml"),
        ("http://schemas.openxmlformats.org/officeDocument/2006/relationships/themeManager", "theme/themeManager.xml")))
    return entries


def valid_masters(entries):
    """`full_deck` holds empty masters, which is enough for rows of the presentation part. These decks must validate."""
    tree = f'<p:cSld><p:spTree>{TREE}</p:spTree></p:cSld>'
    clrmap = ('<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" '
              'accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>')
    entries = replace(entries, "ppt/notesMasters/notesMaster1.xml", f"{DECL}<p:notesMaster {NS}>{tree}{clrmap}</p:notesMaster>")
    return replace(entries, "ppt/handoutMasters/handoutMaster1.xml", f"{DECL}<p:handoutMaster {NS}>{tree}{clrmap}</p:handoutMaster>")


def fixtures():
    yield "generated/package-every-variant.pptx", zipped(valid_masters(package_every_variant())), "open"
    yield "generated/dsig-signed.pptx", zipped(signed_deck()), "open CS-FND-0009"
    yield "generated/mce-preserve.pptx", zipped(mce_preserve_deck()), "open"
    for i in range(len(values("p", "ST_SlideSizeType"))):
        yield f"generated/pres-values-{i + 1:02}.pptx", zipped(valid_masters(pres_values_deck(i))), "open"
    yield "generated/shape-every-construct.pptx", zipped(shape_every_construct()), "open"
    yield "generated/theme-every-construct.pptx", zipped(theme_every_construct()), "open"
