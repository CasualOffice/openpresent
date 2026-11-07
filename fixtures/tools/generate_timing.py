#!/usr/bin/env python3
"""Tier 1I fixtures: every time node, behaviour, condition, target and build; every
transition, in `mc:AlternateContent` with both branches; and hostile timing. **Synthetic:**
written to the schema, not by PowerPoint, so they show we follow the schema and keep what
is there, not what Office does with it. What real decks hold is in fixtures/real.

Imported by generate.py, which writes them and lists them in the manifest. The expected
values are in crates/casual-slide-import/tests/timing.rs and
crates/casual-slide-export/tests/timing.rs.
"""
import re

from generate import DECL, NS, TREE, deck, replace, seeded, strict, zipped
from generate_presets import enum

MC = 'xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"'
P14 = 'xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"'
P15 = 'xmlns:p15="http://schemas.microsoft.com/office/powerpoint/2012/main"'
P159 = 'xmlns:p159="http://schemas.microsoft.com/office/powerpoint/2015/09/main"'
RGB = '<a:srgbClr val="FF0000"/>'
SCHEME = '<a:schemeClr val="accent1"><a:lumMod val="50000"/></a:schemeClr>'
EXT = ('<p:extLst><p:ext uri="{BB962C8B-B14F-4D97-AF65-F5344CB8AC3E}"><p14:creationId '
       'xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" val="1"/></p:ext></p:extLst>')
WAV = b"RIFF\x24\x00\x00\x00WAVEfmt "
pml = lambda n: enum(n, "pml")  # noqa: E731
dml = lambda n: enum(n, "dml-main")  # noqa: E731


def shape(sid, name="s"):
    return (f'<p:sp><p:nvSpPr><p:cNvPr id="{sid}" name="{name} {sid}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
            f'<p:spPr><a:xfrm><a:off x="{sid * 91440}" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
            '<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>t</a:t></a:r></a:p></p:txBody></p:sp>')


SHAPES = "".join(shape(i) for i in (2, 3, 4, 5))


def slide(tail, root_attrs=""):
    return (f'{DECL}<p:sld {NS} {MC}{root_attrs}><p:cSld><p:spTree>{TREE}{SHAPES}</p:spTree></p:cSld>'
            f'<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{tail}</p:sld>')


class Ids:
    def __init__(self):
        self.next = 1

    def __call__(self):
        self.next += 1
        return self.next


def targets():
    """Every child of tgtEl, and every child of spTgt."""
    return ['<p:sldTgt/>',
            '<p:sndTgt r:embed="rId2" name="ding.wav"/>',
            '<p:spTgt spid="2"/>',
            '<p:spTgt spid="2"><p:bg/></p:spTgt>',
            '<p:spTgt spid="2"><p:subSp spid="_x0000_s1"/></p:spTgt>',
            *[f'<p:spTgt spid="3"><p:oleChartEl type="{v}" lvl="{i}"/></p:spTgt>' for i, v in enumerate(pml("ST_TLChartSubelementType"))],
            '<p:spTgt spid="3"><p:txEl/></p:spTgt>',
            '<p:spTgt spid="3"><p:txEl><p:charRg st="0" end="1"/></p:txEl></p:spTgt>',
            '<p:spTgt spid="3"><p:txEl><p:pRg st="0" end="0"/></p:txEl></p:spTgt>',
            *[f'<p:spTgt spid="4"><p:graphicEl><a:dgm id="{{00000000-0000-0000-0000-00000000000{i}}}" bldStep="{v}"/></p:graphicEl></p:spTgt>'
              for i, v in enumerate(dml("ST_DgmBuildStep"))],
            *[f'<p:spTgt spid="4"><p:graphicEl><a:chart seriesIdx="{i}" categoryIdx="{i}" bldStep="{v}"/></p:graphicEl></p:spTgt>'
              for i, v in enumerate(dml("ST_ChartBuildStep"))],
            '<p:inkTgt spid="_x0000_s2"/>']


def conditions(ids):
    """Every event, every trigger, in lists on one node."""
    evts = pml("ST_TLTriggerEvent")
    conds = [f'<p:cond evt="{v}" delay="{i * 10}"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond>' for i, v in enumerate(evts)]
    conds += ['<p:cond delay="indefinite"/>', '<p:cond evt="onEnd"><p:tn val="2"/></p:cond>']
    conds += [f'<p:cond evt="onBegin"><p:rtn val="{v}"/></p:cond>' for v in pml("ST_TLTriggerRuntimeNode")]
    conds += [f'<p:cond evt="onClick"><p:tgtEl>{t}</p:tgtEl></p:cond>' for t in targets()]
    return "".join(conds)


def ctn(ids, attrs="", inner=""):
    return f'<p:cTn id="{ids()}" fill="hold"{attrs}>{inner}</p:cTn>'


def cbhvr(ids, attrs="", spid=2, names="ppt_x"):
    return (f'<p:cBhvr{attrs}><p:cTn id="{ids()}" dur="500"/><p:tgtEl><p:spTgt spid="{spid}"/></p:tgtEl>'
            f'<p:attrNameLst><p:attrName>{names}</p:attrName></p:attrNameLst></p:cBhvr>')


def variants():
    return ['<p:boolVal val="1"/>', '<p:intVal val="-5"/>', '<p:fltVal val="1.5"/>', '<p:strVal val="visible"/>', f'<p:clrVal>{RGB}</p:clrVal>']


def behaviours(ids):
    """Every behaviour, with every value of what it declares."""
    out = []
    additive, accumulate = pml("ST_TLBehaviorAdditiveType"), pml("ST_TLBehaviorAccumulateType")
    xfrm, over = pml("ST_TLBehaviorTransformType"), pml("ST_TLBehaviorOverrideType")
    for i, v in enumerate(additive):
        attrs = (f' additive="{v}" accumulate="{accumulate[i % 2]}" xfrmType="{xfrm[i % 2]}" from="0" to="1" by="1" '
                 f'rctx="PPT" override="{over[i % 2]}"')
        out.append(f'<p:anim to="1">{cbhvr(ids, attrs)}</p:anim>')
    for i, calc in enumerate(pml("ST_TLAnimateBehaviorCalcMode")):
        vtype = pml("ST_TLAnimateBehaviorValueType")[i]
        tavs = "".join(f'<p:tav tm="{j * 10000}" fmla="#ppt_x+1"><p:val>{v}</p:val></p:tav>' for j, v in enumerate(variants()))
        tavs += '<p:tav/><p:tav tm="indefinite"/>'
        out.append(f'<p:anim by="1" from="0" to="1" calcmode="{calc}" valueType="{vtype}">{cbhvr(ids)}<p:tavLst>{tavs}</p:tavLst></p:anim>')
    for space, direction in zip(pml("ST_TLAnimateColorSpace"), pml("ST_TLAnimateColorDirection")):
        out.append(f'<p:animClr clrSpc="{space}" dir="{direction}">{cbhvr(ids, names="fillcolor")}'
                   f'<p:from>{RGB}</p:from><p:to>{SCHEME}</p:to></p:animClr>')
    out.append(f'<p:animClr>{cbhvr(ids, names="fillcolor")}<p:by><p:rgb r="10000" g="-20000" b="30000"/></p:by></p:animClr>')
    out.append(f'<p:animClr>{cbhvr(ids, names="fillcolor")}<p:by><p:hsl h="60000" s="10000" l="-20000"/></p:by></p:animClr>')
    for v in pml("ST_TLAnimateEffectTransition"):
        out.append(f'<p:animEffect transition="{v}" filter="fade" prLst="gradientSize:0.5">{cbhvr(ids)}<p:progress><p:fltVal val="0.5"/></p:progress></p:animEffect>')
    for origin, mode in zip(pml("ST_TLAnimateMotionBehaviorOrigin"), pml("ST_TLAnimateMotionPathEditMode")):
        out.append(f'<p:animMotion origin="{origin}" path="M 0 0 L 0.5 0.5 E" pathEditMode="{mode}" rAng="60000" ptsTypes="AA">{cbhvr(ids)}'
                   '<p:by x="1000" y="2000"/><p:from x="0" y="0"/><p:to x="50000" y="50000"/><p:rCtr x="100" y="-100"/></p:animMotion>')
    out.append(f'<p:animRot by="60000" from="0" to="21600000">{cbhvr(ids, names="r")}</p:animRot>')
    out.append(f'<p:animScale zoomContents="1">{cbhvr(ids)}<p:by x="100000" y="100000"/><p:from x="50000" y="50000"/><p:to x="200000" y="200000"/></p:animScale>')
    for v in pml("ST_TLCommandType"):
        out.append(f'<p:cmd type="{v}" cmd="playFrom(0.0)">{cbhvr(ids)}</p:cmd>')
    for var in variants():
        out.append(f'<p:set>{cbhvr(ids, names="style.visibility")}<p:to>{var}</p:to></p:set>')
    media = (f'<p:cMediaNode vol="80000" mute="1" numSld="2" showWhenStopped="0"><p:cTn id="{{}}" fill="hold"/>'
             '<p:tgtEl><p:spTgt spid="5"/></p:tgtEl></p:cMediaNode>')
    out.append(f'<p:audio isNarration="1">{media.format(ids())}</p:audio>')
    out.append(f'<p:video fullScrn="1">{media.format(ids())}</p:video>')
    return out


def common_attrs():
    """Every attribute of cTn, each value at least once."""
    rows = []
    classes, restarts = pml("ST_TLTimeNodePresetClassType"), pml("ST_TLTimeNodeRestartType")
    fills, syncs = pml("ST_TLTimeNodeFillType"), pml("ST_TLTimeNodeSyncType")
    rels, types = ("sameClick", "nextClick"), pml("ST_TLTimeNodeType")
    for i in range(max(len(classes), len(fills), len(types))):
        rows.append(f' presetID="{i + 1}" presetClass="{classes[i % len(classes)]}" presetSubtype="{i}" dur="{i * 100}" '
                    f'repeatCount="{1000 + i}" repeatDur="{i * 2}" spd="100000" accel="10000" decel="20000" autoRev="1" '
                    f'restart="{restarts[i % len(restarts)]}" fill="{fills[i % len(fills)]}" syncBehavior="{syncs[i % len(syncs)]}" '
                    f'tmFilter="0,0;1,1" evtFilter="cancelBubble" display="0" masterRel="{rels[i % len(rels)]}" bldLvl="1" grpId="0" '
                    f'afterEffect="1" nodeType="{types[i % len(types)]}" nodePh="1"')
    return rows


def every_node():
    ids = Ids()
    kids = []
    for i, attrs in enumerate(common_attrs()):
        iterates = pml("ST_IterateType")
        inner = (f'<p:stCondLst><p:cond delay="0"/></p:stCondLst><p:endCondLst><p:cond evt="end"/></p:endCondLst>'
                 f'<p:endSync evt="onEnd" delay="0"><p:rtn val="all"/></p:endSync>'
                 f'<p:iterate type="{iterates[i % len(iterates)]}" backwards="1"><p:tmAbs val="100"/></p:iterate>'
                 f'<p:childTnLst><p:set>{cbhvr(ids)}<p:to><p:strVal val="visible"/></p:to></p:set></p:childTnLst>'
                 f'<p:subTnLst><p:set>{cbhvr(ids)}<p:to><p:strVal val="hidden"/></p:to></p:set></p:subTnLst>')
        kids.append(f'<p:par><p:cTn id="{ids()}"{attrs}>{inner}</p:cTn></p:par>')
    kids.append(f'<p:excl><p:cTn id="{ids()}"><p:iterate><p:tmPct val="1000"/></p:iterate>'
                f'<p:childTnLst>{"".join(behaviours(ids))}</p:childTnLst></p:cTn></p:excl>')
    kids.append(f'<p:par><p:cTn id="{ids()}"><p:stCondLst>{conditions(ids)}</p:stCondLst></p:cTn></p:par>')
    seqs = "".join(f'<p:seq concurrent="1" prevAc="{p}" nextAc="{n}"><p:cTn id="{ids()}" repeatCount="indefinite"/>'
                   '<p:prevCondLst><p:cond evt="onPrev" delay="0"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:prevCondLst>'
                   '<p:nextCondLst><p:cond evt="onNext" delay="0"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:nextCondLst></p:seq>'
                   for p, n in zip(pml("ST_TLPreviousActionType"), pml("ST_TLNextActionType")))
    root = (f'<p:tnLst><p:par><p:cTn id="1" dur="indefinite" restart="never" nodeType="tmRoot"><p:childTnLst>'
            f'{"".join(kids)}{seqs}</p:childTnLst></p:cTn></p:par></p:tnLst>')
    builds, grp = [], 0

    def grp_id():
        nonlocal grp
        grp += 1
        return grp - 1

    templates = "".join(f'<p:tmpl lvl="{i}"><p:tnLst><p:par><p:cTn id="{ids()}"/></p:par></p:tnLst></p:tmpl>' for i in range(2))
    for v in pml("ST_TLParaBuildType"):
        builds.append(f'<p:bldP spid="3" grpId="{grp_id()}" uiExpand="1" build="{v}" bldLvl="2" animBg="1" autoUpdateAnimBg="0" rev="1" advAuto="1000"/>')
    builds.append(f'<p:bldP spid="3" grpId="{grp_id()}" advAuto="indefinite"><p:tmplLst>{templates}</p:tmplLst></p:bldP>')
    for v in pml("ST_TLDiagramBuildType"):
        builds.append(f'<p:bldDgm spid="4" grpId="{grp_id()}" bld="{v}"/>')
    for v in pml("ST_TLOleChartBuildType"):
        builds.append(f'<p:bldOleChart spid="4" grpId="{grp_id()}" bld="{v}" animBg="0"/>')
    builds.append(f'<p:bldGraphic spid="4" grpId="{grp_id()}"><p:bldAsOne/></p:bldGraphic>')
    for v in enum("ST_AnimationDgmBuildType", "dml-main"):
        builds.append(f'<p:bldGraphic spid="4" grpId="{grp_id()}"><p:bldSub><a:bldDgm bld="{v}" rev="1"/></p:bldSub></p:bldGraphic>')
    for v in enum("ST_AnimationChartBuildType", "dml-main"):
        builds.append(f'<p:bldGraphic spid="4" grpId="{grp_id()}"><p:bldSub><a:bldChart bld="{v}" animBg="0"/></p:bldSub></p:bldGraphic>')
    groups = "".join(f'<p:par><p:cTn id="{ids()}" grpId="{g}"/></p:par>' for g in range(grp))
    root = root.replace("</p:childTnLst></p:cTn></p:par></p:tnLst>", f"{groups}</p:childTnLst></p:cTn></p:par></p:tnLst>")
    return slide(f'<p:timing>{root}<p:bldLst>{"".join(builds)}</p:bldLst>{EXT}</p:timing>')


def ecma_only():
    """What the standard allows and Microsoft's validator refuses (tools/ooxml-sdk), kept
    apart so `timing-every-node` is what both accept (F4-02, owner instruction 25)."""
    ids = Ids()
    effects = "".join(f'<p:animEffect>{cbhvr(ids)}<p:progress>{var}</p:progress></p:animEffect>' for var in variants())
    return slide(f'<p:timing><p:tnLst><p:par><p:cTn id="1" nodeType="tmRoot" masterRel="lastClick"><p:childTnLst>{effects}</p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>')


def sound_deck(wave=WAV):
    return deck(slide_rels=[("audio", "../media/sound1.wav")], parts=[("ppt/media/sound1.wav", wave)],
                types_extra=[("/ppt/media/sound1.wav", "audio/wav")])


def timing_deck():
    return zipped(replace(sound_deck(), "ppt/slides/slide1.xml", every_node()))


def alternate(requires, ns, choice, fallback):
    return (f'<mc:AlternateContent><mc:Choice {ns} Requires="{requires}">{choice}</mc:Choice>'
            f'<mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>')


def alternate_timing():
    """As PowerPoint writes newer effects: a branch inside a child list, a whole timing in a
    branch, and a transition and timing that share nothing."""
    set_ = lambda i, spid: (f'<p:set><p:cBhvr><p:cTn id="{i}" dur="1" fill="hold"/><p:tgtEl><p:spTgt spid="{spid}"/></p:tgtEl>'  # noqa: E731
                            '<p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="visible"/></p:to></p:set>')
    inner = alternate("p14", P14, f'<p:par><p:cTn id="3" fill="hold" p14:presetBounceEnd="50000"><p:childTnLst>{set_(4, 2)}</p:childTnLst></p:cTn></p:par>',
                      f'<p:par><p:cTn id="5" fill="hold"><p:childTnLst>{set_(6, 3)}</p:childTnLst></p:cTn></p:par>')
    one = f'<p:timing><p:tnLst><p:par><p:cTn id="1" nodeType="tmRoot"><p:childTnLst><p:par><p:cTn id="2"><p:childTnLst>{inner}</p:childTnLst></p:cTn></p:par></p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>'
    whole = alternate("p14", P14,
                      f'<p:timing><p:tnLst><p:par><p:cTn id="1" nodeType="tmRoot"><p:childTnLst>{set_(2, 2)}</p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>',
                      f'<p:timing><p:tnLst><p:par><p:cTn id="1" nodeType="tmRoot"><p:childTnLst>{set_(2, 4)}</p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>')
    entries = deck(slides=2)
    entries = replace(entries, "ppt/slides/slide1.xml", slide(one))
    return zipped(replace(entries, "ppt/slides/slide2.xml", slide(whole)))


def newer_effects():
    """Every effect from the 2010 namespace, `prstTrans` and Morph, as attribute values."""
    side, lr = pml("ST_TransitionSideDirectionType"), ("l", "r")
    cases = [f'<p14:vortex dir="{v}"/>' for v in side]
    cases += [f'<p14:{n} dir="{v}"/>' for n in ("switch", "flip", "ferris", "gallery", "conveyor") for v in lr]
    cases += [f'<p14:ripple dir="{v}"/>' for v in ("center", "lu", "ru", "ld", "rd")]
    cases += ['<p14:honeycomb/>', '<p14:flash/>']
    cases += [f'<p14:prism dir="{v}" isContent="1" isInverted="1"/>' for v in side]
    cases += [f'<p14:{n} dir="{v}"/>' for n in ("doors", "window") for v in ("horz", "vert")]
    cases += [f'<p14:pan dir="{v}"/>' for v in side]
    cases += [f'<p14:glitter dir="{v}" pattern="{p}"/>' for v, p in zip(side, ("diamond", "hexagon", "diamond", "hexagon"))]
    cases += [f'<p14:warp dir="{v}"/>' for v in ("in", "out")]
    cases += [f'<p14:flythrough dir="{v}" hasBounce="1"/>' for v in ("in", "out")]
    cases += [f'<p14:shred pattern="{p}" dir="{v}"/>' for p, v in zip(("strip", "rectangle"), ("in", "out"))]
    cases += [f'<p14:reveal thruBlk="1" dir="{v}"/>' for v in lr]
    cases += ['<p14:wheelReverse spokes="4"/>']
    out = [("p14", P14, f'<p:transition spd="slow" p14:dur="{1000 + i}" advTm="{i}">{c}</p:transition>') for i, c in enumerate(cases)]
    out += [("p15", P15, f'<p:transition spd="slow"><p15:prstTrans prst="{n}" invX="1" invY="1"/></p:transition>')
            for n in ("fallOver", "drape", "curtains", "wind", "prestige", "fracture", "crush", "peelOff", "pageCurlDouble", "pageCurlSingle", "airplane", "origami")]
    out += [("p159", P159, f'<p:transition spd="slow"><p159:morph option="{v}"/></p:transition>') for v in ("byObject", "byWord", "byChar")]
    return out


def transitions_deck():
    """Each slide: one newer effect in a branch, an older fallback, sound and extLst."""
    cases = newer_effects()
    sound = ('<p:sndAc><p:stSnd loop="1"><p:snd r:embed="rId2" name="ding.wav"/></p:stSnd></p:sndAc>', '<p:sndAc><p:endSnd/></p:sndAc>')
    entries = deck(slides=len(cases) + 2, parts=[("ppt/media/sound1.wav", WAV)], types_extra=[("/ppt/media/sound1.wav", "audio/wav")])
    media = '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/audio" Target="../media/sound1.wav"/>'
    out = entries
    for i, (req, ns, choice) in enumerate(cases, 1):
        fallback = f'<p:transition spd="slow"><p:fade thruBlk="1"/></p:transition>'
        out = replace(out, f"ppt/slides/slide{i}.xml", slide(alternate(req, ns, choice, fallback)))
    i = len(cases) + 1
    out = replace(out, f"ppt/slides/slide{i}.xml", slide(f'<p:transition spd="med" advClick="0" advTm="1500"><p:push dir="u"/>{sound[0]}{EXT}</p:transition>'))
    out = replace(out, f"ppt/slides/slide{i + 1}.xml", slide(f'<p:transition><p:wipe/>{sound[1]}</p:transition>'))
    out = [(n, d.replace("</Relationships>", media + "</Relationships>") if n in (f"ppt/slides/_rels/slide{i}.xml.rels",) else d) for n, d in out]
    return zipped(out)


def strict_timing():
    """Strict's `N%` forms where its schema has nothing else. Namespaces by mapping alone."""
    def percent(match):
        value = int(match.group(2))
        whole, part = divmod(abs(value), 1000)
        text = f"{whole}.{part:03}".rstrip("0").rstrip(".")
        return f'{match.group(1)}="{"-" if value < 0 else ""}{text}%"'
    attrs = r'\b(spd|accel|decel|vol|tm|r|g|b|s|l|x|y)="(-?\d+)"'
    head, tail = every_node().split("<p:timing>")
    tail = re.sub(attrs, percent, tail)
    part = head + "<p:timing>" + re.sub(r'(<p:tmPct val|<a:lumMod val)="(\d+)"', percent, tail)
    return zipped(strict(replace(sound_deck(WAV.decode("latin1")), "ppt/slides/slide1.xml", part)))


def with_tail(tail):
    return zipped(replace(sound_deck(), "ppt/slides/slide1.xml", slide(tail)))


def root_of(kids, bld=""):
    return (f'<p:timing><p:tnLst><p:par><p:cTn id="1" nodeType="tmRoot"><p:childTnLst>{kids}</p:childTnLst></p:cTn></p:par></p:tnLst>{bld}</p:timing>')


def hostile():
    """Each opens with the part read, the slide's timing left unread or the value reported.
    Codes are in crates/casual-slide-import/tests/timing.rs."""
    deep = '<p:par><p:cTn><p:childTnLst>' * 32 + '</p:childTnLst></p:cTn></p:par>' * 32
    yield "hostile/timing-limit-depth.bin", with_tail(root_of(deep)), "open"
    empty_set = '<p:set/>'
    yield "hostile/timing-limit-nodes.bin", with_tail(root_of(seeded(empty_set, 100_000))), "open"
    cond = '<p:cond/>'
    yield "hostile/timing-limit-conditions.bin", with_tail(root_of(f'<p:par><p:cTn><p:stCondLst>{seeded(cond, 4097)}</p:stCondLst></p:cTn></p:par>')), "open"
    name = '<p:attrName>x</p:attrName>'
    yield "hostile/timing-limit-attr-names.bin", with_tail(root_of(f'<p:set><p:cBhvr><p:attrNameLst>{seeded(name, 1025)}</p:attrNameLst></p:cBhvr></p:set>')), "open"
    tav = '<p:tav/>'
    yield "hostile/timing-limit-animate-values.bin", with_tail(root_of(f'<p:anim><p:tavLst>{seeded(tav, 65537)}</p:tavLst></p:anim>')), "open"
    build = '<p:bldP spid="2" grpId="0"/>'
    yield "hostile/timing-limit-builds.bin", with_tail(root_of("", f'<p:bldLst>{seeded(build, 65537)}</p:bldLst>')), "open"
    template = '<p:tmpl/>'
    yield "hostile/timing-limit-templates.bin", with_tail(root_of("", f'<p:bldLst><p:bldP spid="2" grpId="0"><p:tmplLst>{template * 65}</p:tmplLst></p:bldP></p:bldLst>')), "open"
    branch = '<mc:Fallback/>'
    yield "hostile/timing-limit-branches.bin", with_tail(root_of(f'<mc:AlternateContent>{branch * 65}</mc:AlternateContent>')), "open"
    yield "hostile/timing-long-value.bin", with_tail(root_of(f'<p:cmd cmd="{"x" * (1 << 16)}"/>') + f'<p:transition p14:dur="{"1" * (1 << 16)}" {P14}/>'), "open"
    bad = ('<p:par/><p:par><p:cTn id="x" presetID="z" presetClass="z" presetSubtype="z" dur="-1" repeatCount="z" repeatDur="z" spd="z" accel="200%" decel="-1%" autoRev="z" '
           'restart="z" fill="z" syncBehavior="z" display="z" masterRel="z" bldLvl="z" grpId="-1" afterEffect="z" nodeType="z" nodePh="z" unknown="1"><p:foreign/><p:stCondLst/>'
           '<p:stCondLst/><p:endSync/><p:endSync/><p:iterate type="z" backwards="z"><p:tmAbs/><p:tmPct val="-1"/></p:iterate>'
           '<p:childTnLst><p:cmd type="z"/><p:video/><p:audio/><p:anim calcmode="z" valueType="z"><p:foreign/></p:anim><p:foreign/>'
           '<p:set><p:cBhvr additive="z" accumulate="z" xfrmType="z" override="z"><p:tgtEl><p:sldTgt x="1"><p:foreign/></p:sldTgt><p:inkTgt/></p:tgtEl><p:attrNameLst><p:attrName><p:foreign/></p:attrName></p:attrNameLst></p:cBhvr>'
           '<p:to><p:boolVal val="z"/></p:to></p:set><p:animClr clrSpc="z" dir="z"><p:by><p:rgb r="200%" g="z"/><p:hsl/></p:by><p:from/><p:to><a:srgbClr val="nope"/></p:to></p:animClr>'
           '<p:animMotion origin="z" pathEditMode="z" rAng="z"><p:by x="z"/></p:animMotion><p:animRot by="z"/><p:animScale zoomContents="z"><p:to/></p:animScale>'
           '<p:animEffect transition="z"><p:progress><p:fltVal val="zz"/></p:progress></p:animEffect></p:childTnLst></p:cTn></p:par>'
           '<p:seq concurrent="z" prevAc="z" nextAc="z"><p:prevCondLst/><p:nextCondLst><p:cond evt="z" delay="z"><p:tn val="z"/><p:rtn val="z"/></p:cond></p:nextCondLst></p:seq>'
           '<p:set><p:cBhvr><p:cTn id="9"/><p:tgtEl><p:spTgt spid="z"><p:oleChartEl type="z" lvl="z"/><p:graphicEl><a:chart seriesIdx="z" bldStep="z"/><a:dgm bldStep="z"/></p:graphicEl></p:spTgt></p:tgtEl></p:cBhvr></p:set>')
    builds = ('<p:bldLst><p:bldP spid="z" grpId="z" build="z" animBg="z"/><p:bldDgm bld="z"/><p:bldOleChart bld="z"/><p:bldGraphic spid="2" grpId="0"><p:bldSub><a:bldChart bld="z"/><a:bldDgm bld="z"/></p:bldSub></p:bldGraphic><p:foreign/></p:bldLst>')
    yield "hostile/timing-bad-values.bin", with_tail(root_of(bad, builds)), "open"
    dangling = (f'<p:set><p:cBhvr><p:cTn id="5"/><p:tgtEl><p:spTgt spid="99"/></p:tgtEl></p:cBhvr></p:set><p:set><p:cBhvr><p:cTn id="5"/><p:tgtEl><p:sndTgt r:embed="rId99"/></p:tgtEl></p:cBhvr></p:set>'
                '<p:par><p:cTn id="6" grpId="7"><p:stCondLst><p:cond><p:tn val="404"/></p:cond></p:stCondLst></p:cTn></p:par>')
    bld = '<p:bldLst><p:bldP spid="98" grpId="0"/><p:bldP spid="98" grpId="0"/></p:bldLst>'
    yield "hostile/timing-dangling.bin", with_tail(root_of(dangling, bld)), "open"
    yield "hostile/timing-not-one-par.bin", with_tail('<p:timing><p:tnLst><p:set/><p:par/></p:tnLst><p:tnLst/><p:foreign/></p:timing><p:timing/>'), "open"
    transition = ('<p:transition spd="z" advClick="z" advTm="-1"><p:foreign/><p:cover dir="z"/><p:fade/><p:wheel spokes="z"/><p:split orient="z" dir="z"/>'
                  '<p:sndAc><p:stSnd loop="z"/><p:endSnd/></p:sndAc><p:sndAc/></p:transition><p:transition/>')
    yield "hostile/transition-bad-values.bin", with_tail(transition), "open"
    out_of_order = ('<mc:AlternateContent><mc:Fallback><p:transition><p:fade/></p:transition></mc:Fallback><mc:Choice Requires="p14"><p:transition/></mc:Choice>'
                    '<p:foreign/></mc:AlternateContent>')
    yield "hostile/transition-alternate-disorder.bin", with_tail(out_of_order), "open"


def fixtures():
    yield "generated/timing-every-node.pptx", timing_deck(), "open"
    yield "generated/timing-ecma-only.pptx", zipped(replace(deck(), "ppt/slides/slide1.xml", ecma_only())), "open"
    yield "generated/timing-alternate-content.pptx", alternate_timing(), "open"
    yield "generated/transition-every-effect.pptx", transitions_deck(), "open"
    yield "generated/timing-strict.pptx", strict_timing(), "open"
    yield from hostile()
