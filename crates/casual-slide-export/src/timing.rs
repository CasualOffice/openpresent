//! Patch `p:timing` and `p:transition` into a part's tree (ADR-013, Tier 1I).
//!
//! The model, before and after, becomes a description of the elements it stands for, and
//! `crate::drawing::patch` touches only what differs. So an unedited tree changes nothing,
//! spelling included, and what the model does not hold stays where it is.
//!
//! `mc:AlternateContent` keeps its branches: an edit to one branch touches that branch
//! alone. The model holds both as values, so a caller sees the other, and this writer
//! does not guess at keeping them alike (`BEH-25` is about what cannot be seen).
//! What the writer refuses, with nothing saved:
//!
//! - `CS-EXP-0009`: asking for what was left unread, a new extension effect outside a
//!   branch requiring its namespace, or a branch list changing inside an
//!   `mc:AlternateContent` that holds something else too;
//! - `CS-EXP-0008`: a tree built by hand deeper than the reader would believe;
//! - `CS-SLE-0005`: a new target naming a `spid` on no shape, as PowerPoint requires
//!   one (`MS-OI29500` 2.1.1176 a, 2.1.1149 a). Renaming a shape's `spid` without its
//!   targets is that too: `SlidePart::retarget_shape` moves both.
//!
//! The rest of range is the reader's: `patch_slide_part` reads the part back.

use std::collections::HashSet;

use casual_slide_import::properties::{Elem, ns};
use casual_slide_import::timing::{MC, TimingLimits, shape_ids};
use casual_slide_model::color::{Angle, Percentage as ColorPercentage};
use casual_slide_model::presentation::Percentage;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::SlidePart;
use casual_slide_model::timing::*;
use casual_slide_model::transition::SlideTransition;
use casual_slide_ooxml::XmlTree;

use crate::drawing::{Node, Space, color_node, insert_new, patch};
use crate::properties::{ExportError, Writer};
use crate::transition::{effect_token, transition_node};

type Result<T = ()> = std::result::Result<T, ExportError>;

/// Where a child goes among its siblings, for what this tier adds to the schema's order.
pub(crate) fn rank(parent: &str, local: &str) -> Option<usize> {
    let order: &[&str] = match parent {
        "timing" => &["tnLst", "bldLst", "extLst"],
        "cTn" => &[
            "stCondLst",
            "endCondLst",
            "endSync",
            "iterate",
            "childTnLst",
            "subTnLst",
        ],
        "seq" => &["cTn", "prevCondLst", "nextCondLst"],
        "cBhvr" => &["cTn", "tgtEl", "attrNameLst"],
        "anim" => &["cBhvr", "tavLst"],
        "animClr" | "animScale" => &["cBhvr", "by", "from", "to"],
        "animEffect" => &["cBhvr", "progress"],
        "animMotion" => &["cBhvr", "by", "from", "to", "rCtr"],
        "set" => &["cBhvr", "to"],
        "cMediaNode" => &["cTn", "tgtEl"],
        // Any effect, then these: `transition`'s effects are one choice.
        "transition" => {
            return Some(match local {
                "sndAc" => 1,
                "extLst" => 2,
                _ => 0,
            });
        }
        _ => return None,
    };
    order.iter().position(|o| *o == local)
}

pub(crate) fn p_node(local: &'static str) -> Node {
    Node::new(Space::Pml, local)
}

fn pct(value: Option<Percentage>) -> Option<ColorPercentage> {
    value.map(|p| ColorPercentage(p.0))
}

fn holder(local: &'static str, kid: Option<Node>) -> Node {
    p_node(local).opt(kid)
}

/// `mc:AlternateContent`, with `xmlns` for what `Requires` names and this writer knows.
// spec-audit: mc:Choice/@Requires
fn alternate_node<T>(alternates: &Alternates<T>, kids: impl Fn(&T) -> Vec<Node>) -> Node {
    Node::new(Space::Mc, "AlternateContent").list(alternates.branches.iter().map(|branch| {
        let local = if branch.is_choice {
            "Choice"
        } else {
            "Fallback"
        };
        let mut node = Node::new(Space::Mc, local);
        for token in branch.requires.iter().flat_map(|r| requires_tokens(r)) {
            node = match token {
                "p14" => node.lex(
                    "xmlns:p14",
                    Some(&casual_slide_import::transition::P14[0].to_owned()),
                ),
                "p15" => node.lex(
                    "xmlns:p15",
                    Some(&casual_slide_import::transition::P15[0].to_owned()),
                ),
                "p159" => node.lex(
                    "xmlns:p159",
                    Some(&casual_slide_import::transition::P159[0].to_owned()),
                ),
                _ => node,
            };
        }
        node.lex("Requires", branch.requires.as_ref())
            .list(kids(&branch.value))
    }))
}

// spec-audit: owner=sndTgt,snd,tgtEl,stSnd
pub(crate) fn sound_node(local: &'static str, s: &EmbeddedSound) -> Node {
    p_node(local)
        .rel("embed", s.rel_id.as_ref())
        .lex("name", s.name.as_ref())
        .lex("builtIn", s.built_in.as_ref())
}

// spec-audit: owner=charRg,pRg,txEl
fn range_node(local: &'static str, r: &IndexRange) -> Node {
    p_node(local)
        .lex("st", r.start.as_ref())
        .lex("end", r.end.as_ref())
}

// spec-audit: owner=dgm,graphicEl
fn dgm_element_node(d: &AnimationDiagramElement) -> Node {
    Node::a("dgm")
        .lex("id", d.id.as_ref())
        .lex("bldStep", d.build_step.as_ref())
}

// spec-audit: owner=chart,graphicEl
fn chart_element_node(c: &AnimationChartElement) -> Node {
    Node::a("chart")
        .lex("seriesIdx", c.series_idx.as_ref())
        .lex("categoryIdx", c.category_idx.as_ref())
        .lex("bldStep", c.build_step.as_ref())
}

// spec-audit: owner=spTgt,tgtEl
fn shape_target_node(t: &ShapeTarget) -> Node {
    let sub = t.sub.as_ref().map(|sub| match sub {
        ShapeSubTarget::Background => p_node("bg"),
        ShapeSubTarget::SubShape(spid) => sub_spid_node("subSp", spid.as_ref()),
        ShapeSubTarget::OleChart(c) => ole_chart_node(c),
        ShapeSubTarget::Text(text) => p_node("txEl").opt(match text {
            TextTarget::Whole => None,
            TextTarget::Characters(r) => Some(range_node("charRg", r)),
            TextTarget::Paragraphs(r) => Some(range_node("pRg", r)),
        }),
        ShapeSubTarget::Graphic(g) => p_node("graphicEl").opt(match g {
            AnimationElementChoice::Empty => None,
            AnimationElementChoice::Diagram(d) => Some(dgm_element_node(d)),
            AnimationElementChoice::Chart(c) => Some(chart_element_node(c)),
        }),
    });
    p_node("spTgt").lex("spid", t.spid.as_ref()).opt(sub)
}

// spec-audit: owner=subSp,inkTgt,spTgt,tgtEl
fn sub_spid_node(local: &'static str, spid: Option<&String>) -> Node {
    p_node(local).lex("spid", spid)
}

// spec-audit: owner=oleChartEl,spTgt
fn ole_chart_node(c: &OleChartTarget) -> Node {
    p_node("oleChartEl")
        .lex("type", c.element_type.as_ref())
        .lex("lvl", c.lvl.as_ref())
}

// spec-audit: owner=tgtEl,cond,endSync,cBhvr,cMediaNode
fn target_node(t: &TimeTarget) -> Node {
    p_node("tgtEl").opt(match t {
        TimeTarget::Empty => None,
        TimeTarget::Slide => Some(p_node("sldTgt")),
        TimeTarget::Sound(s) => Some(sound_node("sndTgt", s)),
        TimeTarget::Shape(s) => Some(shape_target_node(s)),
        TimeTarget::Ink(spid) => Some(sub_spid_node("inkTgt", spid.as_ref())),
    })
}

// spec-audit: owner=cond,endSync,cTn,stCondLst,endCondLst,prevCondLst,nextCondLst,tn,rtn
fn condition_node(local: &'static str, c: &TimeCondition) -> Node {
    p_node(local)
        .lex("evt", c.evt.as_ref())
        .lex("delay", c.delay.as_ref())
        .opt(c.trigger.as_ref().map(|t| match t {
            ConditionTrigger::Target(t) => target_node(t),
            ConditionTrigger::Node(val) => p_node("tn").lex("val", val.as_ref()),
            ConditionTrigger::Runtime(val) => p_node("rtn").lex("val", val.as_ref()),
        }))
}

// spec-audit: owner=stCondLst,endCondLst,prevCondLst,nextCondLst,cTn,seq
fn conditions_node(local: &'static str, list: &[TimeCondition]) -> Node {
    p_node(local).list(list.iter().map(|c| condition_node("cond", c)))
}

// spec-audit: owner=iterate,cTn,tmAbs,tmPct
fn iterate_node(i: &IterateData) -> Node {
    p_node("iterate")
        .lex("type", i.iterate_type.as_ref())
        .lex("backwards", i.backwards.as_ref())
        .opt(i.interval.as_ref().map(|interval| match interval {
            IterateInterval::Time(val) => p_node("tmAbs").lex("val", val.as_ref()),
            IterateInterval::Percent(val) => p_node("tmPct").pct("val", pct(*val)),
        }))
}

// spec-audit: owner=tnLst,childTnLst,subTnLst,cTn,tmpl,timing
fn list_node(local: &'static str, nodes: &[TimeNode]) -> Node {
    p_node(local).list(nodes.iter().map(time_node))
}

// spec-audit: owner=cTn,par,excl,seq,cBhvr,cMediaNode
fn common_node(c: &CommonTimeNode) -> Node {
    p_node("cTn")
        .lex("id", c.node_id.as_ref())
        .lex("presetID", c.preset_id.as_ref())
        .lex("presetClass", c.preset_class.as_ref())
        .lex("presetSubtype", c.preset_subtype.as_ref())
        .lex("dur", c.dur.as_ref())
        .lex("repeatCount", c.repeat_count.as_ref())
        .lex("repeatDur", c.repeat_dur.as_ref())
        .pct("spd", pct(c.spd))
        .pct("accel", pct(c.accel))
        .pct("decel", pct(c.decel))
        .lex("autoRev", c.auto_rev.as_ref())
        .lex("restart", c.restart.as_ref())
        .lex("fill", c.fill.as_ref())
        .lex("syncBehavior", c.sync_behavior.as_ref())
        .lex("tmFilter", c.tm_filter.as_ref())
        .lex("evtFilter", c.evt_filter.as_ref())
        .lex("display", c.display.as_ref())
        .lex("masterRel", c.master_rel.as_ref())
        .lex("bldLvl", c.bld_lvl.as_ref())
        .lex("grpId", c.grp_id.as_ref())
        .lex("afterEffect", c.after_effect.as_ref())
        .lex("nodeType", c.node_type.as_ref())
        .lex("nodePh", c.node_ph.as_ref())
        .opt(
            c.st_cond_lst
                .as_deref()
                .map(|l| conditions_node("stCondLst", l)),
        )
        .opt(
            c.end_cond_lst
                .as_deref()
                .map(|l| conditions_node("endCondLst", l)),
        )
        .opt(c.end_sync.as_ref().map(|s| condition_node("endSync", s)))
        .opt(c.iterate.as_ref().map(iterate_node))
        .opt(
            c.child_tn_lst
                .as_deref()
                .map(|l| list_node("childTnLst", l)),
        )
        .opt(c.sub_tn_lst.as_deref().map(|l| list_node("subTnLst", l)))
}

// spec-audit: owner=cBhvr,anim,animClr,animEffect,animMotion,animRot,animScale,cmd,set,attrNameLst
fn behavior_node(b: &CommonBehavior) -> Node {
    p_node("cBhvr")
        .lex("additive", b.additive.as_ref())
        .lex("accumulate", b.accumulate.as_ref())
        .lex("xfrmType", b.xfrm_type.as_ref())
        .lex("from", b.from.as_ref())
        .lex("to", b.to.as_ref())
        .lex("by", b.by.as_ref())
        .lex("rctx", b.rctx.as_ref())
        .lex("override", b.override_style.as_ref())
        .opt(b.common.as_ref().map(common_node))
        .opt(b.target.as_ref().map(target_node))
        .opt(b.attr_name_lst.as_deref().map(|names| {
            p_node("attrNameLst").list(
                names
                    .iter()
                    .map(|n| p_node("attrName").text(Some(n.clone()))),
            )
        }))
}

// spec-audit: owner=to,val,progress,set,tav,animEffect,boolVal,intVal,fltVal,strVal,clrVal
fn variant_node(local: &'static str, v: &AnimVariant) -> Node {
    p_node(local).opt(match v {
        AnimVariant::Empty => None,
        AnimVariant::Bool(val) => Some(p_node("boolVal").lex("val", val.as_ref())),
        AnimVariant::Int(val) => Some(p_node("intVal").lex("val", val.as_ref())),
        AnimVariant::Float(val) => Some(p_node("fltVal").lex("val", val.as_ref())),
        AnimVariant::Str(val) => Some(p_node("strVal").lex("val", val.as_ref())),
        AnimVariant::Color(val) => Some(p_node("clrVal").opt(val.as_ref().map(color_node))),
    })
}

// spec-audit: owner=tav,tavLst
fn tav_node(t: &TimeAnimateValue) -> Node {
    let node = match t.tm {
        // A percentage, so spelled as Strict requires.
        Some(AnimateValueTime::At(at)) => p_node("tav").pct("tm", pct(Some(at))),
        other => p_node("tav").lex("tm", other.as_ref()),
    };
    node.lex("fmla", t.fmla.as_ref())
        .opt(t.val.as_ref().map(|v| variant_node("val", v)))
}

// spec-audit: owner=by,from,to,rCtr,animMotion,animScale
fn point_node(local: &'static str, p: &TimePoint) -> Node {
    p_node(local).pct("x", pct(p.x)).pct("y", pct(p.y))
}

// spec-audit: owner=rgb,by
fn by_rgb_node(by: &ByRgb) -> Node {
    p_node("rgb")
        .pct("r", pct(by.red))
        .pct("g", pct(by.green))
        .pct("b", pct(by.blue))
}

// spec-audit: owner=hsl,by
fn by_hsl_node(by: &ByHsl) -> Node {
    p_node("hsl")
        .ang("h", by.hue)
        .pct("s", pct(by.saturation))
        .pct("l", pct(by.lightness))
}

// spec-audit: owner=cMediaNode,audio,video
fn media_node(m: &CommonMediaNode) -> Node {
    p_node("cMediaNode")
        .pct("vol", pct(m.vol))
        .lex("mute", m.mute.as_ref())
        .lex("numSld", m.num_sld.as_ref())
        .lex("showWhenStopped", m.show_when_stopped.as_ref())
        .opt(m.common.as_ref().map(common_node))
        .opt(m.target.as_ref().map(target_node))
}

fn bhvr(b: &Option<CommonBehavior>) -> Option<Node> {
    b.as_ref().map(behavior_node)
}

// spec-audit: owner=by,from,to
fn angle_attrs(node: Node, by: Option<Angle>, from: Option<Angle>, to: Option<Angle>) -> Node {
    node.ang("by", by).ang("from", from).ang("to", to)
}

// spec-audit: owner=par,excl,seq,anim,animClr,animEffect,animMotion,animRot,animScale,cmd,set,audio,video,tnLst,childTnLst,subTnLst
fn time_node(node: &TimeNode) -> Node {
    match node {
        TimeNode::Par(n) => holder("par", n.common.as_ref().map(common_node)),
        TimeNode::Excl(n) => holder("excl", n.common.as_ref().map(common_node)),
        TimeNode::Seq(n) => p_node("seq")
            .lex("concurrent", n.concurrent.as_ref())
            .lex("prevAc", n.prev_ac.as_ref())
            .lex("nextAc", n.next_ac.as_ref())
            .opt(n.common.as_ref().map(common_node))
            .opt(
                n.prev_cond_lst
                    .as_deref()
                    .map(|l| conditions_node("prevCondLst", l)),
            )
            .opt(
                n.next_cond_lst
                    .as_deref()
                    .map(|l| conditions_node("nextCondLst", l)),
            ),
        TimeNode::Anim(n) => p_node("anim")
            .lex("by", n.by.as_ref())
            .lex("from", n.from.as_ref())
            .lex("to", n.to.as_ref())
            .lex("calcmode", n.calcmode.as_ref())
            .lex("valueType", n.value_type.as_ref())
            .opt(bhvr(&n.behavior))
            .opt(
                n.tav_lst
                    .as_deref()
                    .map(|l| p_node("tavLst").list(l.iter().map(tav_node))),
            ),
        TimeNode::AnimClr(n) => p_node("animClr")
            .lex("clrSpc", n.clr_spc.as_ref())
            .lex("dir", n.dir.as_ref())
            .opt(bhvr(&n.behavior))
            .opt(n.by.as_ref().map(|by| {
                p_node("by").opt(match by {
                    ByAnimateColor::Empty => None,
                    ByAnimateColor::Rgb(rgb) => Some(by_rgb_node(rgb)),
                    ByAnimateColor::Hsl(hsl) => Some(by_hsl_node(hsl)),
                })
            }))
            .opt(
                n.from
                    .as_ref()
                    .map(|c| p_node("from").opt(c.as_ref().map(color_node))),
            )
            .opt(
                n.to.as_ref()
                    .map(|c| p_node("to").opt(c.as_ref().map(color_node))),
            ),
        TimeNode::AnimEffect(n) => p_node("animEffect")
            .lex("transition", n.transition.as_ref())
            .lex("filter", n.filter.as_ref())
            .lex("prLst", n.pr_lst.as_ref())
            .opt(bhvr(&n.behavior))
            .opt(n.progress.as_ref().map(|v| variant_node("progress", v))),
        TimeNode::AnimMotion(n) => p_node("animMotion")
            .lex("origin", n.origin.as_ref())
            .lex("path", n.path.as_ref())
            .lex("pathEditMode", n.path_edit_mode.as_ref())
            .ang("rAng", n.r_ang)
            .lex("ptsTypes", n.pts_types.as_ref())
            .opt(bhvr(&n.behavior))
            .opt(n.by.as_ref().map(|p| point_node("by", p)))
            .opt(n.from.as_ref().map(|p| point_node("from", p)))
            .opt(n.to.as_ref().map(|p| point_node("to", p)))
            .opt(n.r_ctr.as_ref().map(|p| point_node("rCtr", p))),
        TimeNode::AnimRot(n) => {
            angle_attrs(p_node("animRot"), n.by, n.from, n.to).opt(bhvr(&n.behavior))
        }
        TimeNode::AnimScale(n) => p_node("animScale")
            .lex("zoomContents", n.zoom_contents.as_ref())
            .opt(bhvr(&n.behavior))
            .opt(n.by.as_ref().map(|p| point_node("by", p)))
            .opt(n.from.as_ref().map(|p| point_node("from", p)))
            .opt(n.to.as_ref().map(|p| point_node("to", p))),
        TimeNode::Cmd(n) => p_node("cmd")
            .lex("type", n.command_type.as_ref())
            .lex("cmd", n.cmd.as_ref())
            .opt(bhvr(&n.behavior)),
        TimeNode::Set(n) => p_node("set")
            .opt(bhvr(&n.behavior))
            .opt(n.to.as_ref().map(|v| variant_node("to", v))),
        TimeNode::Audio(n) => p_node("audio")
            .lex("isNarration", n.is_narration.as_ref())
            .opt(n.media.as_ref().map(media_node)),
        TimeNode::Video(n) => p_node("video")
            .lex("fullScrn", n.full_scrn.as_ref())
            .opt(n.media.as_ref().map(media_node)),
        TimeNode::Alternate(a) => alternate_node(a, |nodes| nodes.iter().map(time_node).collect()),
    }
}

// spec-audit: owner=bldP,bldDgm,bldOleChart,bldGraphic
fn with_build_target(node: Node, t: &BuildTarget) -> Node {
    node.lex("spid", t.spid.as_ref())
        .lex("grpId", t.grp_id.as_ref())
        .lex("uiExpand", t.ui_expand.as_ref())
}

// spec-audit: owner=bldDgm,bldSub
fn dgm_build_node(d: &DiagramBuildProperties) -> Node {
    Node::a("bldDgm")
        .lex("bld", d.bld.as_ref())
        .lex("rev", d.rev.as_ref())
}

// spec-audit: owner=bldChart,bldSub
fn chart_build_node(c: &ChartBuildProperties) -> Node {
    Node::a("bldChart")
        .lex("bld", c.bld.as_ref())
        .lex("animBg", c.anim_bg.as_ref())
}

// spec-audit: owner=bldLst,bldP,bldDgm,bldOleChart,bldGraphic,bldSub,tmplLst
fn build_node(entry: &BuildEntry) -> Node {
    match entry {
        BuildEntry::Paragraph(b) => with_build_target(p_node("bldP"), &b.target)
            .lex("build", b.build.as_ref())
            .lex("bldLvl", b.bld_lvl.as_ref())
            .lex("animBg", b.anim_bg.as_ref())
            .lex("autoUpdateAnimBg", b.auto_update_anim_bg.as_ref())
            .lex("rev", b.rev.as_ref())
            .lex("advAuto", b.adv_auto.as_ref())
            .opt(
                b.tmpl_lst
                    .as_deref()
                    .map(|list| p_node("tmplLst").list(list.iter().map(template_node))),
            ),
        BuildEntry::Diagram(b) => {
            with_build_target(p_node("bldDgm"), &b.target).lex("bld", b.bld.as_ref())
        }
        BuildEntry::OleChart(b) => with_build_target(p_node("bldOleChart"), &b.target)
            .lex("bld", b.bld.as_ref())
            .lex("animBg", b.anim_bg.as_ref()),
        BuildEntry::Graphic(b) => with_build_target(p_node("bldGraphic"), &b.target).opt(
            b.choice.as_ref().map(|c| match c {
                GraphicBuildChoice::AsOne => p_node("bldAsOne"),
                GraphicBuildChoice::Sub(sub) => p_node("bldSub").opt(match sub {
                    GraphicalObjectBuildProperties::Empty => None,
                    GraphicalObjectBuildProperties::Diagram(d) => Some(dgm_build_node(d)),
                    GraphicalObjectBuildProperties::Chart(c) => Some(chart_build_node(c)),
                }),
            }),
        ),
    }
}

// spec-audit: owner=tmpl,tmplLst
fn template_node(t: &BuildTemplate) -> Node {
    p_node("tmpl")
        .lex("lvl", t.lvl.as_ref())
        .opt(t.tn_lst.as_deref().map(|l| list_node("tnLst", l)))
}

/// `p:CT_SlideTiming`.
// spec-audit: owner=timing
pub(crate) fn timing_node(t: &SlideTiming) -> Node {
    p_node("timing")
        .opt(t.tn_lst.as_deref().map(|l| list_node("tnLst", l)))
        .opt(
            t.bld_lst
                .as_deref()
                .map(|l| p_node("bldLst").list(l.iter().map(build_node))),
        )
}

fn unsupported(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    }
}

/// `name` in a branch of `el`, or the direct child.
fn direct_child(w: &Writer<'_>, parent: &Elem, name: &str) -> Result<Option<Elem>> {
    Ok(parent.first_child(w.tree, w.part, ns::PML, name)?)
}

fn branches_of(w: &Writer<'_>, ac: &Elem) -> Result<Vec<Elem>> {
    Ok(ac
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(MC, "Choice") || c.is(MC, "Fallback"))
        .collect())
}

/// The `mc:AlternateContent` at the root that holds `name`.
fn holding_alternate(w: &Writer<'_>, root: &Elem, name: &str) -> Result<Option<Elem>> {
    for child in root.children(w.tree, w.part)? {
        if child.is(MC, "AlternateContent") {
            for branch in branches_of(w, &child)? {
                if direct_child(w, &branch, name)?.is_some() {
                    return Ok(Some(child));
                }
            }
        }
    }
    Ok(None)
}

/// Whether a branch of this holds anything other than `name`.
fn holds_more(w: &Writer<'_>, ac: &Elem, name: &str) -> Result<bool> {
    for branch in branches_of(w, ac)? {
        for child in branch.children(w.tree, w.part)? {
            if !child.is(ns::PML, name) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Every element of `name` at the root goes, and an `mc:AlternateContent` holding it.
fn remove_all(w: &mut Writer<'_>, root: &Elem, name: &str) -> Result {
    while let Some(ac) = holding_alternate(w, root, name)? {
        if holds_more(w, &ac, name)? {
            return Err(unsupported(w, "alternate content holding more than this"));
        }
        w.tree.remove_child(root.id, ac.id);
        w.changes += 1;
    }
    w.remove_all(root, ns::PML, name)
}

/// The place of what a root child holds, for `mc:AlternateContent` what is in its first
/// branch that has anything.
fn order_of(w: &Writer<'_>, order: &[&str], child: &Elem) -> Result<Option<usize>> {
    if child.is(MC, "AlternateContent") {
        for branch in branches_of(w, child)? {
            if let Some(first) = branch.children(w.tree, w.part)?.first() {
                return Ok(order.iter().position(|o| *o == first.local));
            }
        }
        return Ok(None);
    }
    if child.in_namespace(ns::PML) {
        return Ok(order.iter().position(|o| *o == child.local));
    }
    Ok(None)
}

fn insert_at_root(
    w: &mut Writer<'_>,
    root: &Elem,
    order: &[&str],
    name: &str,
    node: &Node,
) -> Result {
    let mine = order.iter().position(|o| *o == name);
    let mut anchor = None;
    for child in root.children(w.tree, w.part)? {
        if let (Some(theirs), Some(mine)) = (order_of(w, order, &child)?, mine)
            && theirs > mine
        {
            anchor = Some(child);
            break;
        }
    }
    insert_new(w, root, anchor.as_ref(), node)
}

fn slot_node<T>(slot: &Slot<T>, make: fn(&T) -> Node) -> Option<Node> {
    match slot {
        Slot::Plain(value) => Some(make(value)),
        Slot::Alternate(a) => Some(alternate_node(a, |value| {
            value.iter().map(|v| make(v)).collect()
        })),
        Slot::Unread => None,
    }
}

/// Patch one of `p:transition` and `p:timing`.
fn patch_slot<T: PartialEq>(
    w: &mut Writer<'_>,
    root: &Elem,
    order: &[&str],
    name: &'static str,
    before: Option<&Slot<T>>,
    after: Option<&Slot<T>>,
    make: fn(&T) -> Node,
) -> Result {
    if before == after {
        return Ok(());
    }
    match (before, after) {
        (_, Some(Slot::Unread)) => Err(unsupported(w, "timing or a transition left unread")),
        (Some(Slot::Plain(b)), Some(Slot::Plain(a))) => {
            let Some(el) = direct_child(w, root, name)? else {
                return Err(ExportError::MalformedElement {
                    part: w.part.to_owned(),
                });
            };
            patch(w, &el, Some(&make(b)), &make(a))
        }
        (Some(Slot::Alternate(b)), Some(Slot::Alternate(a)))
            if b.branches.len() == a.branches.len()
                && b.branches
                    .iter()
                    .zip(&a.branches)
                    .all(|(x, y)| x.is_choice == y.is_choice) =>
        {
            let Some(ac) = holding_alternate(w, root, name)? else {
                return Err(ExportError::MalformedElement {
                    part: w.part.to_owned(),
                });
            };
            let elements = branches_of(w, &ac)?;
            for ((old, new), branch) in b.branches.iter().zip(&a.branches).zip(&elements) {
                w.attr(
                    branch,
                    "Requires",
                    old.requires.as_ref(),
                    new.requires.as_ref(),
                )?;
                match (&old.value, &new.value) {
                    (Some(x), Some(y)) if x == y => {}
                    (Some(x), Some(y)) => {
                        let Some(el) = direct_child(w, branch, name)? else {
                            return Err(ExportError::MalformedElement {
                                part: w.part.to_owned(),
                            });
                        };
                        patch(w, &el, Some(&make(x)), &make(y))?;
                    }
                    (None, Some(y)) => insert_new(w, branch, None, &make(y))?,
                    (Some(_), None) => w.remove_all(branch, ns::PML, name)?,
                    (None, None) => {}
                }
            }
            Ok(())
        }
        (_, None) => remove_all(w, root, name),
        (_, Some(slot)) => {
            remove_all(w, root, name)?;
            match slot_node(slot, make) {
                Some(node) => insert_at_root(w, root, order, name, &node),
                None => Ok(()),
            }
        }
    }
}

/// Whether `token` is in what this branch requires.
fn requires(branch: &AlternateBranch<Option<Box<SlideTransition>>>, token: &str) -> bool {
    branch
        .requires
        .iter()
        .flat_map(|r| requires_tokens(r))
        .any(|t| t == token)
}

/// What a transition needs its branch to require.
fn tokens_needed(t: &SlideTransition) -> Vec<&'static str> {
    t.effect
        .as_ref()
        .and_then(effect_token)
        .into_iter()
        .chain(t.p14_dur.as_ref().map(|_| "p14"))
        .collect()
}

/// A new effect from another namespace has to be where a file may hold it, or the
/// deck is not valid and PowerPoint can refuse it. What a file already holds stays.
fn check_transition(
    w: &Writer<'_>,
    before: Option<&Slot<SlideTransition>>,
    after: Option<&Slot<SlideTransition>>,
) -> Result {
    const WRONG: &str = "a newer transition outside a branch requiring its namespace";
    match after {
        Some(Slot::Plain(a)) => {
            let same =
                matches!(before, Some(Slot::Plain(b)) if tokens_needed(b) == tokens_needed(a));
            if !same && !tokens_needed(a).is_empty() {
                return Err(unsupported(w, WRONG));
            }
        }
        Some(Slot::Alternate(a)) => {
            for (i, branch) in a.branches.iter().enumerate() {
                let old = match before {
                    Some(Slot::Alternate(b)) => b.branches.get(i),
                    _ => None,
                };
                if old == Some(branch) {
                    continue;
                }
                let Some(value) = &branch.value else { continue };
                if tokens_needed(value).iter().any(|t| !requires(branch, t)) {
                    return Err(unsupported(w, WRONG));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn known_token(token: &str) -> bool {
    matches!(token, "p14" | "p15" | "p159")
}

fn tokens_of<T>(slot: Option<&Slot<T>>) -> Vec<&str> {
    match slot {
        Some(Slot::Alternate(a)) => a
            .branches
            .iter()
            .flat_map(|b| b.requires.iter().flat_map(|r| requires_tokens(r)))
            .collect(),
        _ => Vec::new(),
    }
}

/// A new `Requires` needs its prefix declared, or the branch is never selected.
fn check_requires<T>(
    w: &Writer<'_>,
    root: &Elem,
    before: Option<&Slot<T>>,
    after: Option<&Slot<T>>,
) -> Result {
    let held = tokens_of(before);
    for token in tokens_of(after) {
        if !held.contains(&token) && !known_token(token) && root.scope.uri(token).is_none() {
            return Err(unsupported(
                w,
                "a branch requiring a prefix nothing declares",
            ));
        }
    }
    Ok(())
}

/// Patch a part's transition and timing.
pub(crate) fn patch_motion(
    w: &mut Writer<'_>,
    root: &Elem,
    order: &[&str],
    before: &SlidePart,
    after: &SlidePart,
    limits: &TimingLimits,
) -> Result {
    if before.transition != after.transition {
        check_transition(w, before.transition.as_ref(), after.transition.as_ref())?;
        check_requires(
            w,
            root,
            before.transition.as_ref(),
            after.transition.as_ref(),
        )?;
        patch_slot(
            w,
            root,
            order,
            "transition",
            before.transition.as_ref(),
            after.transition.as_ref(),
            transition_node,
        )?;
    }
    if before.timing != after.timing {
        let deep = |t: &Slot<SlideTiming>| match t {
            Slot::Plain(value) => !value.depth_within(limits.max_node_depth),
            Slot::Alternate(a) => a.branches.iter().any(|b| {
                b.value
                    .as_ref()
                    .is_some_and(|v| !v.depth_within(limits.max_node_depth))
            }),
            Slot::Unread => false,
        };
        if after.timing.as_ref().is_some_and(deep) {
            return Err(ExportError::VariantTooDeep {
                part: w.part.to_owned(),
            });
        }
        check_requires(w, root, before.timing.as_ref(), after.timing.as_ref())?;
        patch_slot(
            w,
            root,
            order,
            "timing",
            before.timing.as_ref(),
            after.timing.as_ref(),
            timing_node,
        )?;
    }
    Ok(())
}

fn model_ids(part: &SlidePart) -> HashSet<u32> {
    part.shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
        .filter_map(|node| node.non_visual.as_ref()?.spid)
        .collect()
}

/// A `spid` the timing names and no shape holds, if this edit makes a new one. A target
/// the file already held dangling stays as it was. A shape in a branch the resolver did
/// not choose is in the tree and not the model, and counts.
pub(crate) fn new_dangling_target(tree: &XmlTree, before: &SlidePart, after: &SlidePart) -> bool {
    let before_ids = model_ids(before);
    let unseen: HashSet<u32> = shape_ids(tree).difference(&before_ids).copied().collect();
    let dangling = |part: &SlidePart, ids: &HashSet<u32>| -> HashSet<u32> {
        part.animated_spids()
            .into_iter()
            .filter(|spid| !ids.contains(spid) && !unseen.contains(spid))
            .collect()
    };
    let was = dangling(before, &before_ids);
    dangling(after, &model_ids(after))
        .difference(&was)
        .next()
        .is_some()
}
