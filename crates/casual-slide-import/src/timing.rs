//! Read `p:timing` from an [`XmlTree`] (Tier 1I, `T1I-01`):
//! every time node, behaviour, condition, target and build list, in the part or in
//! `mc:AlternateContent`, whose every branch is read and whose choice is the resolver's.
//!
//! **Bad timing never loses the deck (owner instruction 24, `SEC-C10`).** A value that
//! cannot be used is reported and left out. Over a bound in [`TimingLimits`], or where
//! the reader itself cannot go on, the whole of `p:timing` reads as
//! [`Slot::Unread`], the tree keeps it, the report says so, and the part still reads.
//! Depth follows the input only as far as `max_node_depth`, and nothing is evaluated:
//! a path, a formula and a command are strings, and nothing here runs them or plays
//! anything. Nothing about what the tree does when it runs is decided here (`BEH-23`).
//!
//! Findings carry element and attribute names only. Commands, formulas and filters are
//! file content and never reach a report.

use std::collections::{HashMap, HashSet};

use casual_slide_model::color::Angle;
use casual_slide_model::presentation::Percentage;
use casual_slide_model::slide::SlidePart;
use casual_slide_model::timing::*;
use casual_slide_ooxml::{AlternateResolution, Branch, ElementId, Relationship, XmlTree};

use crate::color::{Findings as ColorFindings, is_color_element, read_color_choice};
use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, ns};
use crate::transition::P14;

/// `mc:`.
pub const MC: &[&str] = &["http://schemas.openxmlformats.org/markup-compatibility/2006"];

/// Bounds on one part's timing and transition, on top of the XML bounds. Crossing one
/// leaves the whole `p:timing` or `p:transition` unread, in the tree and in the report
/// as `CS-TMG-0001`. Each is far beyond a real deck: `docs/21` says what they cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingLimits {
    /// Time nodes nested in time nodes, `mc:AlternateContent` counting as one.
    pub max_node_depth: usize,
    /// Time nodes in one part's timing.
    pub max_nodes: usize,
    /// Conditions in one list.
    pub max_conditions: usize,
    /// `attrName` in one `attrNameLst`.
    pub max_attr_names: usize,
    /// `tav` in one `tavLst`.
    pub max_animate_values: usize,
    /// Entries in one `bldLst`.
    pub max_builds: usize,
    /// `tmpl` in one `tmplLst`. The schema allows 9.
    pub max_templates: usize,
    /// Branches in one `mc:AlternateContent`.
    pub max_branches: usize,
}

impl Default for TimingLimits {
    fn default() -> Self {
        Self {
            max_node_depth: 32,
            max_nodes: 100_000,
            max_conditions: 4_096,
            max_attr_names: 1_024,
            max_animate_values: 65_536,
            max_builds: 65_536,
            max_templates: 64,
            max_branches: 64,
        }
    }
}

/// What the timing reader noticed, under its own codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimingFinding {
    /// A bound in [`TimingLimits`] was crossed. The element named is what crossed it,
    /// and the whole `p:timing` or `p:transition` stays in the tree, unread.
    OverLimit,
    /// The reader could not go on, as with a value over `ImportLimits::max_value_bytes`.
    /// The whole `p:timing` or `p:transition` stays in the tree, unread.
    Unreadable,
    /// A target or a build names a `spid` on no shape in the part. Kept.
    /// `MS-OI29500` 2.1.1176 a and 2.1.1149 a: PowerPoint requires one.
    UnknownShape,
    /// A `grpId` with no match in `bldLst` (2.1.1157 h), or a `spid` and `grpId` pair
    /// repeated there (2.1.1149 b). Kept.
    BuildMismatch,
    /// `tnLst` not one `par`, which the schema allows and PowerPoint does not
    /// (2.1.1181 a and b). Kept.
    RootNotParallel,
}

impl TimingFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::OverLimit => "CS-TMG-0001",
            Self::Unreadable => "CS-TMG-0002",
            Self::UnknownShape => "CS-TMG-0003",
            Self::BuildMismatch => "CS-TMG-0004",
            Self::RootNotParallel => "CS-TMG-0005",
        }
    }
}

/// The resolver's answers, by element, so a part of many costs one lookup each.
pub(crate) fn branch_index(resolutions: &[AlternateResolution]) -> HashMap<ElementId, Branch> {
    resolutions.iter().map(|a| (a.element, a.branch)).collect()
}

/// Why a timing tree stopped being read.
pub(crate) enum Stop {
    /// A bound, and what crossed it.
    Over(&'static str),
    /// A shared bound, or a fault in the XML.
    Failed,
}

impl From<ImportError> for Stop {
    fn from(_: ImportError) -> Self {
        Self::Failed
    }
}

pub(crate) type Read<T> = Result<T, Stop>;

/// A property that may appear once. A repeat is reported, and the first is used.
macro_rules! once {
    ($self:ident, $child:expr, $slot:expr, $read:expr) => {
        if $slot.is_some() {
            $self.r.note(K::DuplicateElement, $child, None);
        } else {
            $slot = Some($read);
        }
    };
}
pub(crate) use once;

/// One part's timing and transition being read.
pub(crate) struct MotionReader<'a, 'r> {
    pub r: &'a mut Reader<'r>,
    pub rels: &'a [Relationship],
    pub color: &'a mut ColorFindings,
    pub limits: TimingLimits,
    pub alternates: &'a HashMap<ElementId, Branch>,
    pub nodes: usize,
}

fn fixed_positive(p: &Percentage) -> bool {
    (0..=100_000).contains(&p.0)
}

fn fixed(p: &Percentage) -> bool {
    (-100_000..=100_000).contains(&p.0)
}

fn value_time(t: &AnimateValueTime) -> bool {
    match t {
        AnimateValueTime::At(p) => fixed_positive(p),
        AnimateValueTime::Indefinite => true,
    }
}

impl<'a, 'r> MotionReader<'a, 'r> {
    pub fn new(
        r: &'a mut Reader<'r>,
        rels: &'a [Relationship],
        color: &'a mut ColorFindings,
        limits: TimingLimits,
        alternates: &'a HashMap<ElementId, Branch>,
    ) -> Self {
        Self {
            r,
            rels,
            color,
            limits,
            alternates,
            nodes: 0,
        }
    }

    /// Read one child of a part's root, if it is a transition, timing, or an
    /// `mc:AlternateContent` holding either. Returns whether it was.
    pub fn root_child(&mut self, el: &Elem, part: &mut SlidePart) -> bool {
        if el.is(ns::PML, "transition") {
            if part.transition.is_some() {
                self.r.note(K::DuplicateElement, el, None);
            } else {
                part.transition = Some(self.plain(el, Self::transition));
            }
            return true;
        }
        if el.is(ns::PML, "timing") {
            if part.timing.is_some() {
                self.r.note(K::DuplicateElement, el, None);
            } else {
                part.timing = Some(self.plain(el, Self::timing));
            }
            return true;
        }
        if !el.is(MC, "AlternateContent") {
            return false;
        }
        let holds = |this: &Self, name: &str| -> bool {
            el.children(this.r.tree, this.r.part)
                .unwrap_or_default()
                .iter()
                .filter(|b| b.in_namespace(MC))
                .any(|b| {
                    b.first_child(this.r.tree, this.r.part, ns::PML, name)
                        .ok()
                        .flatten()
                        .is_some()
                })
        };
        let (transition, timing) = (holds(self, "transition"), holds(self, "timing"));
        if !transition && !timing {
            return false;
        }
        if transition {
            if part.transition.is_some() {
                self.r.note(K::DuplicateElement, el, None);
            } else {
                part.transition = Some(self.alternate_slot(el, "transition", Self::transition));
            }
        }
        if timing {
            if part.timing.is_some() {
                self.r.note(K::DuplicateElement, el, None);
            } else {
                part.timing = Some(self.alternate_slot(el, "timing", Self::timing));
            }
        }
        true
    }

    fn give_up<T>(&mut self, el: &Elem, stop: Stop) -> Slot<T> {
        match stop {
            Stop::Over(what) => {
                self.r
                    .findings
                    .note(K::Timing(TimingFinding::OverLimit), self.r.part, what, None);
                self.r.note(K::Timing(TimingFinding::OverLimit), el, None);
            }
            Stop::Failed => self.r.note(K::Timing(TimingFinding::Unreadable), el, None),
        }
        Slot::Unread
    }

    fn plain<T>(&mut self, el: &Elem, read: fn(&mut Self, &Elem) -> Read<T>) -> Slot<T> {
        match read(self, el) {
            Ok(value) => Slot::Plain(Box::new(value)),
            Err(stop) => self.give_up(el, stop),
        }
    }

    fn alternate_slot<T>(
        &mut self,
        el: &Elem,
        name: &'static str,
        read: fn(&mut Self, &Elem) -> Read<T>,
    ) -> Slot<T> {
        let found = self.branches(el, |this, branch| {
            let mut found = None;
            for child in this.r.children(branch)? {
                if child.is(ns::PML, name) {
                    once!(this, &child, found, Box::new(read(this, &child)?));
                } else if !child.is(ns::PML, "transition") && !child.is(ns::PML, "timing") {
                    this.r.retained(&child);
                }
            }
            Ok(found)
        });
        match found {
            Ok(alternates) => Slot::Alternate(alternates),
            Err(stop) => self.give_up(el, stop),
        }
    }

    /// Every branch of an `mc:AlternateContent`, and the one the resolver chose. What
    /// is neither branch is reported and stays.
    // spec-audit: mc:Choice/@Requires
    pub(crate) fn branches<T>(
        &mut self,
        el: &Elem,
        mut read: impl FnMut(&mut Self, &Elem) -> Read<T>,
    ) -> Read<Alternates<T>> {
        self.r.finish(el);
        let chosen = self.alternates.get(&el.id).and_then(|branch| match branch {
            Branch::Choice { element, .. } | Branch::Fallback { element } => Some(*element),
            Branch::None => None,
        });
        let mut out = Alternates {
            branches: Vec::new(),
            selected: None,
        };
        for child in self.r.children(el)? {
            let is_choice = child.is(MC, "Choice");
            if !is_choice && !child.is(MC, "Fallback") {
                self.r.retained(&child);
                continue;
            }
            if out.branches.len() >= self.limits.max_branches {
                return Err(Stop::Over("AlternateContent"));
            }
            let mut branch = child.clone();
            let requires = if is_choice {
                self.r.attr::<String>(&mut branch, "Requires")?
            } else {
                None
            };
            self.r.finish(&branch);
            if Some(child.id) == chosen {
                out.selected = Some(out.branches.len());
            }
            let value = read(self, &child)?;
            out.branches.push(AlternateBranch {
                requires,
                is_choice,
                value,
            });
        }
        Ok(out)
    }

    /// `p:CT_SlideTiming`.
    // spec-audit: owner=timing
    fn timing(&mut self, el: &Elem) -> Read<SlideTiming> {
        self.r.finish(el);
        let mut timing = SlideTiming::default();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "tnLst" => {
                    once!(self, &child, timing.tn_lst, self.nodes_of(&child, 0)?);
                    let list = timing.tn_lst.as_deref().unwrap_or_default();
                    if list.len() > 1
                        || list
                            .iter()
                            .any(|n| !matches!(n, TimeNode::Par(_) | TimeNode::Alternate(_)))
                    {
                        self.r
                            .note(K::Timing(TimingFinding::RootNotParallel), &child, None);
                    }
                }
                "bldLst" => once!(self, &child, timing.bld_lst, self.builds(&child)?),
                // `extLst` stays.
                _ => self.r.retained(&child),
            }
        }
        Ok(timing)
    }

    /// `p:CT_TimeNodeList`, under whichever element holds it. `depth` is how many time
    /// nodes hold this list.
    // spec-audit: owner=tnLst,childTnLst,subTnLst
    fn nodes_of(&mut self, el: &Elem, depth: usize) -> Read<Vec<TimeNode>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.r.children(el)? {
            if child.is(MC, "AlternateContent") {
                out.push(self.alternate_nodes(&child, depth)?);
                continue;
            }
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.as_str();
            if !matches!(
                name,
                "par"
                    | "seq"
                    | "excl"
                    | "anim"
                    | "animClr"
                    | "animEffect"
                    | "animMotion"
                    | "animRot"
                    | "animScale"
                    | "cmd"
                    | "set"
                    | "audio"
                    | "video"
            ) {
                self.r.retained(&child);
                continue;
            }
            self.enter(depth)?;
            let below = depth + 1;
            out.push(match name {
                "par" => TimeNode::Par(self.container(&child, below)?),
                "seq" => TimeNode::Seq(self.sequence(&child, below)?),
                "excl" => TimeNode::Excl(self.container(&child, below)?),
                "anim" => TimeNode::Anim(self.anim(&child, below)?),
                "animClr" => TimeNode::AnimClr(self.anim_clr(&child, below)?),
                "animEffect" => TimeNode::AnimEffect(self.anim_effect(&child, below)?),
                "animMotion" => TimeNode::AnimMotion(self.anim_motion(&child, below)?),
                "animRot" => TimeNode::AnimRot(self.anim_rot(&child, below)?),
                "animScale" => TimeNode::AnimScale(self.anim_scale(&child, below)?),
                "cmd" => TimeNode::Cmd(self.cmd(&child, below)?),
                "set" => TimeNode::Set(self.set(&child, below)?),
                "audio" => TimeNode::Audio(self.audio(&child, below)?),
                _ => TimeNode::Video(self.video(&child, below)?),
            });
        }
        Ok(out)
    }

    fn enter(&mut self, depth: usize) -> Read<()> {
        if depth >= self.limits.max_node_depth {
            return Err(Stop::Over("childTnLst"));
        }
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.limits.max_nodes {
            return Err(Stop::Over("tnLst"));
        }
        Ok(())
    }

    fn alternate_nodes(&mut self, el: &Elem, depth: usize) -> Read<TimeNode> {
        self.enter(depth)?;
        let below = depth + 1;
        let alternates = self.branches(el, |this, branch| this.nodes_of(branch, below))?;
        Ok(TimeNode::Alternate(alternates))
    }

    /// `p:CT_TLTimeNodeParallel` and `p:CT_TLTimeNodeExclusive`.
    // spec-audit: owner=par,excl
    fn container(&mut self, el: &Elem, depth: usize) -> Read<ContainerTimeNode> {
        self.r.finish(el);
        let mut node = ContainerTimeNode::default();
        for child in self.r.children(el)? {
            if child.is(ns::PML, "cTn") {
                once!(self, &child, node.common, self.common(&child, depth)?);
            } else {
                self.r.retained(&child);
            }
        }
        if node.common.is_none() {
            self.r.note(K::MissingRequired, el, Some("cTn"));
        }
        Ok(node)
    }

    /// `p:CT_TLTimeNodeSequence`.
    // spec-audit: owner=seq
    fn sequence(&mut self, el: &Elem, depth: usize) -> Read<SequenceTimeNode> {
        let mut e = el.clone();
        let mut node = SequenceTimeNode {
            concurrent: self.r.attr(&mut e, "concurrent")?,
            prev_ac: self.r.attr(&mut e, "prevAc")?,
            next_ac: self.r.attr(&mut e, "nextAc")?,
            ..SequenceTimeNode::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cTn" => once!(self, &child, node.common, self.common(&child, depth)?),
                "prevCondLst" => once!(
                    self,
                    &child,
                    node.prev_cond_lst,
                    self.conditions(&child, "prevCondLst")?
                ),
                "nextCondLst" => once!(
                    self,
                    &child,
                    node.next_cond_lst,
                    self.conditions(&child, "nextCondLst")?
                ),
                _ => self.r.retained(&child),
            }
        }
        if node.common.is_none() {
            self.r.note(K::MissingRequired, el, Some("cTn"));
        }
        Ok(node)
    }

    /// `p:CT_TLCommonTimeNodeData`.
    // spec-audit: owner=cTn
    fn common(&mut self, el: &Elem, depth: usize) -> Read<CommonTimeNode> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut node = CommonTimeNode {
            node_id: r.attr(&mut e, "id")?,
            preset_id: r.attr(&mut e, "presetID")?,
            preset_class: r.attr(&mut e, "presetClass")?,
            preset_subtype: r.attr(&mut e, "presetSubtype")?,
            dur: r.attr(&mut e, "dur")?,
            repeat_count: r.attr(&mut e, "repeatCount")?,
            repeat_dur: r.attr(&mut e, "repeatDur")?,
            spd: r.attr(&mut e, "spd")?,
            accel: r.attr_in(&mut e, "accel", fixed_positive)?,
            decel: r.attr_in(&mut e, "decel", fixed_positive)?,
            auto_rev: r.attr(&mut e, "autoRev")?,
            restart: r.attr(&mut e, "restart")?,
            fill: r.attr(&mut e, "fill")?,
            sync_behavior: r.attr(&mut e, "syncBehavior")?,
            tm_filter: r.attr(&mut e, "tmFilter")?,
            evt_filter: r.attr(&mut e, "evtFilter")?,
            display: r.attr(&mut e, "display")?,
            master_rel: r.attr(&mut e, "masterRel")?,
            bld_lvl: r.attr(&mut e, "bldLvl")?,
            grp_id: r.attr(&mut e, "grpId")?,
            after_effect: r.attr(&mut e, "afterEffect")?,
            node_type: r.attr(&mut e, "nodeType")?,
            node_ph: r.attr(&mut e, "nodePh")?,
            ..CommonTimeNode::default()
        };
        r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "stCondLst" => once!(
                    self,
                    &child,
                    node.st_cond_lst,
                    self.conditions(&child, "stCondLst")?
                ),
                "endCondLst" => once!(
                    self,
                    &child,
                    node.end_cond_lst,
                    self.conditions(&child, "endCondLst")?
                ),
                "endSync" => once!(self, &child, node.end_sync, self.condition(&child)?),
                "iterate" => once!(self, &child, node.iterate, self.iterate(&child)?),
                "childTnLst" => {
                    once!(
                        self,
                        &child,
                        node.child_tn_lst,
                        self.nodes_of(&child, depth)?
                    )
                }
                "subTnLst" => {
                    once!(self, &child, node.sub_tn_lst, self.nodes_of(&child, depth)?)
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(node)
    }

    fn checked_children(&mut self, el: &Elem, limit: usize, what: &'static str) -> Read<Vec<Elem>> {
        if self.r.tree.child_elements(el.id).len() > limit {
            return Err(Stop::Over(what));
        }
        Ok(self.r.children(el)?)
    }

    /// `p:CT_TLTimeConditionList`.
    // spec-audit: owner=stCondLst,endCondLst,prevCondLst,nextCondLst
    fn conditions(&mut self, el: &Elem, what: &'static str) -> Read<Vec<TimeCondition>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.checked_children(el, self.limits.max_conditions, what)? {
            if child.is(ns::PML, "cond") {
                out.push(self.condition(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        if out.is_empty() {
            self.r.note(K::MissingRequired, el, Some("cond"));
        }
        Ok(out)
    }

    /// `p:CT_TLTimeCondition`.
    // spec-audit: owner=cond,endSync,tn,rtn
    fn condition(&mut self, el: &Elem) -> Read<TimeCondition> {
        let mut e = el.clone();
        let mut condition = TimeCondition {
            evt: self.r.attr(&mut e, "evt")?,
            delay: self.r.attr(&mut e, "delay")?,
            trigger: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            let trigger = match child.local.as_str() {
                "tgtEl" => Some(ConditionTrigger::Target(TimeTarget::Empty)),
                "tn" => Some(ConditionTrigger::Node(None)),
                "rtn" => Some(ConditionTrigger::Runtime(None)),
                _ => None,
            };
            if trigger.is_none() {
                self.r.retained(&child);
                continue;
            }
            once!(
                self,
                &child,
                condition.trigger,
                match child.local.as_str() {
                    "tgtEl" => ConditionTrigger::Target(self.target(&child)?),
                    "tn" => {
                        let mut t = child.clone();
                        let val = self.r.required(&mut t, "val", |_: &u32| true)?;
                        self.r.finish(&t);
                        self.nothing_inside(&child)?;
                        ConditionTrigger::Node(val)
                    }
                    _ => {
                        let mut t = child.clone();
                        let val = self
                            .r
                            .required(&mut t, "val", |_: &TriggerRuntimeNode| true)?;
                        self.r.finish(&t);
                        self.nothing_inside(&child)?;
                        ConditionTrigger::Runtime(val)
                    }
                }
            );
        }
        Ok(condition)
    }

    pub(crate) fn nothing_inside(&mut self, el: &Elem) -> Read<()> {
        for child in self.r.children(el)? {
            self.r.retained(&child);
        }
        Ok(())
    }

    fn empty(&mut self, el: &Elem) -> Read<()> {
        self.r.finish(el);
        self.nothing_inside(el)
    }

    /// `p:CT_TLIterateData`.
    // spec-audit: owner=iterate,tmAbs,tmPct
    fn iterate(&mut self, el: &Elem) -> Read<IterateData> {
        let mut e = el.clone();
        let mut data = IterateData {
            iterate_type: self.r.attr(&mut e, "type")?,
            backwards: self.r.attr(&mut e, "backwards")?,
            interval: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            if !child.in_namespace(ns::PML) || !matches!(name, "tmAbs" | "tmPct") {
                self.r.retained(&child);
                continue;
            }
            let mut t = child.clone();
            let interval = if name == "tmAbs" {
                IterateInterval::Time(self.r.required(&mut t, "val", |_: &TimeValue| true)?)
            } else {
                IterateInterval::Percent(self.r.required(&mut t, "val", |p: &Percentage| p.0 >= 0)?)
            };
            self.r.finish(&t);
            self.nothing_inside(&child)?;
            once!(self, &child, data.interval, interval);
        }
        Ok(data)
    }

    /// `p:CT_TLTimeTargetElement`.
    // spec-audit: owner=tgtEl,inkTgt
    fn target(&mut self, el: &Elem) -> Read<TimeTarget> {
        self.r.finish(el);
        let mut target = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "sldTgt" => {
                    self.empty(&child)?;
                    once!(self, &child, target, TimeTarget::Slide);
                }
                "sndTgt" => once!(self, &child, target, TimeTarget::Sound(self.sound(&child)?)),
                "spTgt" => {
                    once!(
                        self,
                        &child,
                        target,
                        TimeTarget::Shape(self.shape_target(&child)?)
                    )
                }
                "inkTgt" => {
                    let mut t = child.clone();
                    let spid = self.r.required(&mut t, "spid", |_: &String| true)?;
                    self.r.finish(&t);
                    self.nothing_inside(&child)?;
                    once!(self, &child, target, TimeTarget::Ink(spid));
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(target.unwrap_or(TimeTarget::Empty))
    }

    /// `a:CT_EmbeddedWAVAudioFile`, wherever it is. Nothing reads the sound.
    // spec-audit: owner=sndTgt,snd
    pub(crate) fn sound(&mut self, el: &Elem) -> Read<EmbeddedSound> {
        let mut e = el.clone();
        let rel_id = match e.name_of_ns(ns::REL, "embed").map(str::to_owned) {
            Some(written) => self.r.attr::<String>(&mut e, &written)?,
            None => {
                self.r.note(K::MissingRequired, el, Some("embed"));
                None
            }
        };
        if let Some(id) = &rel_id
            && !self.rels.iter().any(|rel| rel.id == *id)
        {
            self.r.note(K::DanglingRelationship, el, Some("embed"));
        }
        let sound = EmbeddedSound {
            rel_id,
            name: self.r.attr(&mut e, "name")?,
            built_in: self.r.attr(&mut e, "builtIn")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(sound)
    }

    /// `p:CT_TLShapeTargetElement`.
    // spec-audit: owner=spTgt,subSp
    fn shape_target(&mut self, el: &Elem) -> Read<ShapeTarget> {
        let mut e = el.clone();
        let mut target = ShapeTarget {
            spid: self.r.required(&mut e, "spid", |_: &u32| true)?,
            sub: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "bg" => {
                    self.empty(&child)?;
                    once!(self, &child, target.sub, ShapeSubTarget::Background);
                }
                "subSp" => {
                    let mut t = child.clone();
                    let spid = self.r.required(&mut t, "spid", |_: &String| true)?;
                    self.r.finish(&t);
                    self.nothing_inside(&child)?;
                    once!(self, &child, target.sub, ShapeSubTarget::SubShape(spid));
                }
                "oleChartEl" => {
                    let chart = self.ole_chart_target(&child)?;
                    once!(self, &child, target.sub, ShapeSubTarget::OleChart(chart));
                }
                "txEl" => {
                    let text = self.text_target(&child)?;
                    once!(self, &child, target.sub, ShapeSubTarget::Text(text));
                }
                "graphicEl" => {
                    let graphic = self.graphic_target(&child)?;
                    once!(self, &child, target.sub, ShapeSubTarget::Graphic(graphic));
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(target)
    }

    /// `p:CT_TLOleChartTargetElement`.
    // spec-audit: owner=oleChartEl
    fn ole_chart_target(&mut self, el: &Elem) -> Read<OleChartTarget> {
        let mut e = el.clone();
        let target = OleChartTarget {
            element_type: self
                .r
                .required(&mut e, "type", |_: &ChartSubelementType| true)?,
            lvl: self.r.attr(&mut e, "lvl")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(target)
    }

    /// `p:CT_TLTextTargetElement`.
    // spec-audit: owner=txEl
    fn text_target(&mut self, el: &Elem) -> Read<TextTarget> {
        self.r.finish(el);
        let mut target = None;
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            if !child.in_namespace(ns::PML) || !matches!(name, "charRg" | "pRg") {
                self.r.retained(&child);
                continue;
            }
            let range = self.index_range(&child)?;
            once!(
                self,
                &child,
                target,
                if name == "charRg" {
                    TextTarget::Characters(range)
                } else {
                    TextTarget::Paragraphs(range)
                }
            );
        }
        Ok(target.unwrap_or(TextTarget::Whole))
    }

    /// `p:CT_IndexRange`.
    // spec-audit: owner=charRg,pRg
    fn index_range(&mut self, el: &Elem) -> Read<IndexRange> {
        let mut e = el.clone();
        let range = IndexRange {
            start: self.r.required(&mut e, "st", |_: &u32| true)?,
            end: self.r.required(&mut e, "end", |_: &u32| true)?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(range)
    }

    /// `a:CT_AnimationElementChoice`.
    // spec-audit: owner=graphicEl
    fn graphic_target(&mut self, el: &Elem) -> Read<AnimationElementChoice> {
        self.r.finish(el);
        let mut choice = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "dgm" => {
                    let dgm = self.dgm_element(&child)?;
                    once!(self, &child, choice, AnimationElementChoice::Diagram(dgm));
                }
                "chart" => {
                    let chart = self.chart_element(&child)?;
                    once!(self, &child, choice, AnimationElementChoice::Chart(chart));
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(choice.unwrap_or(AnimationElementChoice::Empty))
    }

    /// `a:CT_AnimationDgmElement`.
    // spec-audit: owner=dgm
    fn dgm_element(&mut self, el: &Elem) -> Read<AnimationDiagramElement> {
        let mut e = el.clone();
        let element = AnimationDiagramElement {
            id: self.r.attr(&mut e, "id")?,
            build_step: self.r.attr(&mut e, "bldStep")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(element)
    }

    /// `a:CT_AnimationChartElement`.
    // spec-audit: owner=chart
    fn chart_element(&mut self, el: &Elem) -> Read<AnimationChartElement> {
        let mut e = el.clone();
        let element = AnimationChartElement {
            series_idx: self.r.attr(&mut e, "seriesIdx")?,
            category_idx: self.r.attr(&mut e, "categoryIdx")?,
            build_step: self
                .r
                .required(&mut e, "bldStep", |_: &ChartBuildStep| true)?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(element)
    }

    /// `p:CT_TLCommonBehaviorData`.
    // spec-audit: owner=cBhvr
    fn behavior(&mut self, el: &Elem, depth: usize) -> Read<CommonBehavior> {
        let mut e = el.clone();
        let mut behavior = CommonBehavior {
            additive: self.r.attr(&mut e, "additive")?,
            accumulate: self.r.attr(&mut e, "accumulate")?,
            xfrm_type: self.r.attr(&mut e, "xfrmType")?,
            from: self.r.attr(&mut e, "from")?,
            to: self.r.attr(&mut e, "to")?,
            by: self.r.attr(&mut e, "by")?,
            rctx: self.r.attr(&mut e, "rctx")?,
            override_style: self.r.attr(&mut e, "override")?,
            ..CommonBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cTn" => once!(self, &child, behavior.common, self.common(&child, depth)?),
                "tgtEl" => once!(self, &child, behavior.target, self.target(&child)?),
                "attrNameLst" => {
                    once!(
                        self,
                        &child,
                        behavior.attr_name_lst,
                        self.attr_names(&child)?
                    )
                }
                _ => self.r.retained(&child),
            }
        }
        if behavior.common.is_none() {
            self.r.note(K::MissingRequired, el, Some("cTn"));
        }
        if behavior.target.is_none() {
            self.r.note(K::MissingRequired, el, Some("tgtEl"));
        }
        Ok(behavior)
    }

    /// `p:CT_TLBehaviorAttributeNameList`.
    // spec-audit: owner=attrNameLst
    fn attr_names(&mut self, el: &Elem) -> Read<Vec<String>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.checked_children(el, self.limits.max_attr_names, "attrNameLst")? {
            if child.is(ns::PML, "attrName") {
                self.r.finish(&child);
                out.push(self.r.text(&child)?.unwrap_or_default());
            } else {
                self.r.retained(&child);
            }
        }
        if out.is_empty() {
            self.r.note(K::MissingRequired, el, Some("attrName"));
        }
        Ok(out)
    }

    /// `p:CT_TLAnimVariant`, under whichever element holds it.
    // spec-audit: owner=to,val,progress,boolVal,intVal,fltVal,strVal,clrVal
    fn variant(&mut self, el: &Elem) -> Read<AnimVariant> {
        self.r.finish(el);
        let mut value = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            let mut t = child.clone();
            let read = match child.local.as_str() {
                "boolVal" => AnimVariant::Bool(self.r.required(&mut t, "val", |_: &bool| true)?),
                "intVal" => AnimVariant::Int(self.r.required(&mut t, "val", |_: &i32| true)?),
                "fltVal" => AnimVariant::Float(self.r.required(
                    &mut t,
                    "val",
                    |_: &casual_slide_model::properties::Float32| true,
                )?),
                "strVal" => AnimVariant::Str(self.r.required(&mut t, "val", |_: &String| true)?),
                "clrVal" => AnimVariant::Color(self.color_in(&child)?),
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            self.r.finish(&t);
            if !matches!(read, AnimVariant::Color(_)) {
                self.nothing_inside(&child)?;
            }
            once!(self, &child, value, read);
        }
        Ok(value.unwrap_or(AnimVariant::Empty))
    }

    /// `a:CT_Color`, under whichever element holds it. The colour's own reader reports on
    /// the colour; what is beside it is reported here.
    fn color_in(&mut self, el: &Elem) -> Read<Option<casual_slide_model::color::Color>> {
        self.r.finish(el);
        for child in self.r.children(el)? {
            if !is_color_element(self.r.tree, child.id) {
                self.r.retained(&child);
            }
        }
        Ok(read_color_choice(self.r.tree, el.id, self.color))
    }

    /// `p:CT_TLAnimateBehavior`.
    // spec-audit: owner=anim
    fn anim(&mut self, el: &Elem, depth: usize) -> Read<AnimateBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateBehavior {
            by: self.r.attr(&mut e, "by")?,
            from: self.r.attr(&mut e, "from")?,
            to: self.r.attr(&mut e, "to")?,
            calcmode: self.r.attr(&mut e, "calcmode")?,
            value_type: self.r.attr(&mut e, "valueType")?,
            ..AnimateBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, anim.behavior, self.behavior(&child, depth)?),
                "tavLst" => once!(self, &child, anim.tav_lst, self.tav_lst(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if anim.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(anim)
    }

    /// `p:CT_TLTimeAnimateValueList`.
    // spec-audit: owner=tavLst
    fn tav_lst(&mut self, el: &Elem) -> Read<Vec<TimeAnimateValue>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.checked_children(el, self.limits.max_animate_values, "tavLst")? {
            if child.is(ns::PML, "tav") {
                out.push(self.tav(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        Ok(out)
    }

    /// `p:CT_TLTimeAnimateValue`.
    // spec-audit: owner=tav
    fn tav(&mut self, el: &Elem) -> Read<TimeAnimateValue> {
        let mut e = el.clone();
        let mut tav = TimeAnimateValue {
            tm: self.r.attr_in(&mut e, "tm", value_time)?,
            fmla: self.r.attr(&mut e, "fmla")?,
            val: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if child.is(ns::PML, "val") {
                once!(self, &child, tav.val, self.variant(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        Ok(tav)
    }

    /// `p:CT_TLAnimateColorBehavior`.
    // spec-audit: owner=animClr
    fn anim_clr(&mut self, el: &Elem, depth: usize) -> Read<AnimateColorBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateColorBehavior {
            clr_spc: self.r.attr(&mut e, "clrSpc")?,
            dir: self.r.attr(&mut e, "dir")?,
            ..AnimateColorBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, anim.behavior, self.behavior(&child, depth)?),
                "by" => once!(self, &child, anim.by, self.by_color(&child)?),
                "from" => once!(self, &child, anim.from, self.color_in(&child)?),
                "to" => once!(self, &child, anim.to, self.color_in(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if anim.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(anim)
    }

    /// `p:CT_TLByAnimateColorTransform`.
    // spec-audit: owner=by
    fn by_color(&mut self, el: &Elem) -> Read<ByAnimateColor> {
        self.r.finish(el);
        let mut by = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "rgb" => {
                    let rgb = self.by_rgb(&child)?;
                    once!(self, &child, by, ByAnimateColor::Rgb(rgb));
                }
                "hsl" => {
                    let hsl = self.by_hsl(&child)?;
                    once!(self, &child, by, ByAnimateColor::Hsl(hsl));
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(by.unwrap_or(ByAnimateColor::Empty))
    }

    /// `p:CT_TLByRgbColorTransform`.
    // spec-audit: owner=rgb
    fn by_rgb(&mut self, el: &Elem) -> Read<ByRgb> {
        let mut e = el.clone();
        let by = ByRgb {
            red: self.r.required(&mut e, "r", fixed)?,
            green: self.r.required(&mut e, "g", fixed)?,
            blue: self.r.required(&mut e, "b", fixed)?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(by)
    }

    /// `p:CT_TLByHslColorTransform`.
    // spec-audit: owner=hsl
    fn by_hsl(&mut self, el: &Elem) -> Read<ByHsl> {
        let mut e = el.clone();
        let by = ByHsl {
            hue: self.r.required(&mut e, "h", |_: &i32| true)?.map(Angle),
            saturation: self.r.required(&mut e, "s", fixed)?,
            lightness: self.r.required(&mut e, "l", fixed)?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(by)
    }

    /// `p:CT_TLAnimateEffectBehavior`.
    // spec-audit: owner=animEffect
    fn anim_effect(&mut self, el: &Elem, depth: usize) -> Read<AnimateEffectBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateEffectBehavior {
            transition: self.r.attr(&mut e, "transition")?,
            filter: self.r.attr(&mut e, "filter")?,
            pr_lst: self.r.attr(&mut e, "prLst")?,
            ..AnimateEffectBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, anim.behavior, self.behavior(&child, depth)?),
                "progress" => once!(self, &child, anim.progress, self.variant(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if anim.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(anim)
    }

    /// `p:CT_TLPoint`.
    // spec-audit: owner=by,from,to,rCtr
    fn point(&mut self, el: &Elem) -> Read<TimePoint> {
        let mut e = el.clone();
        let point = TimePoint {
            x: self.r.required(&mut e, "x", |_: &Percentage| true)?,
            y: self.r.required(&mut e, "y", |_: &Percentage| true)?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(point)
    }

    /// `p:CT_TLAnimateMotionBehavior`.
    // spec-audit: owner=animMotion
    fn anim_motion(&mut self, el: &Elem, depth: usize) -> Read<AnimateMotionBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateMotionBehavior {
            origin: self.r.attr(&mut e, "origin")?,
            path: self.r.attr(&mut e, "path")?,
            path_edit_mode: self.r.attr(&mut e, "pathEditMode")?,
            r_ang: self.r.attr::<i32>(&mut e, "rAng")?.map(Angle),
            pts_types: self.r.attr(&mut e, "ptsTypes")?,
            ..AnimateMotionBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, anim.behavior, self.behavior(&child, depth)?),
                "by" => once!(self, &child, anim.by, self.point(&child)?),
                "from" => once!(self, &child, anim.from, self.point(&child)?),
                "to" => once!(self, &child, anim.to, self.point(&child)?),
                "rCtr" => once!(self, &child, anim.r_ctr, self.point(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if anim.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(anim)
    }

    /// `p:CT_TLAnimateRotationBehavior`.
    // spec-audit: owner=animRot
    fn anim_rot(&mut self, el: &Elem, depth: usize) -> Read<AnimateRotationBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateRotationBehavior {
            by: self.r.attr::<i32>(&mut e, "by")?.map(Angle),
            from: self.r.attr::<i32>(&mut e, "from")?.map(Angle),
            to: self.r.attr::<i32>(&mut e, "to")?.map(Angle),
            behavior: None,
        };
        self.r.finish(&e);
        anim.behavior = self.only_behavior(el, depth)?;
        Ok(anim)
    }

    fn only_behavior(&mut self, el: &Elem, depth: usize) -> Read<Option<CommonBehavior>> {
        let mut behavior = None;
        for child in self.r.children(el)? {
            if child.is(ns::PML, "cBhvr") {
                once!(self, &child, behavior, self.behavior(&child, depth)?);
            } else {
                self.r.retained(&child);
            }
        }
        if behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(behavior)
    }

    /// `p:CT_TLAnimateScaleBehavior`.
    // spec-audit: owner=animScale
    fn anim_scale(&mut self, el: &Elem, depth: usize) -> Read<AnimateScaleBehavior> {
        let mut e = el.clone();
        let mut anim = AnimateScaleBehavior {
            zoom_contents: self.r.attr(&mut e, "zoomContents")?,
            ..AnimateScaleBehavior::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, anim.behavior, self.behavior(&child, depth)?),
                "by" => once!(self, &child, anim.by, self.point(&child)?),
                "from" => once!(self, &child, anim.from, self.point(&child)?),
                "to" => once!(self, &child, anim.to, self.point(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if anim.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(anim)
    }

    /// `p:CT_TLCommandBehavior`. The command is a string, and nothing runs it.
    // spec-audit: owner=cmd
    fn cmd(&mut self, el: &Elem, depth: usize) -> Read<CommandBehavior> {
        let mut e = el.clone();
        let mut command = CommandBehavior {
            command_type: self.r.attr(&mut e, "type")?,
            cmd: self.r.attr(&mut e, "cmd")?,
            behavior: None,
        };
        self.r.finish(&e);
        command.behavior = self.only_behavior(el, depth)?;
        Ok(command)
    }

    /// `p:CT_TLSetBehavior`.
    // spec-audit: owner=set
    fn set(&mut self, el: &Elem, depth: usize) -> Read<SetBehavior> {
        self.r.finish(el);
        let mut set = SetBehavior::default();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cBhvr" => once!(self, &child, set.behavior, self.behavior(&child, depth)?),
                "to" => once!(self, &child, set.to, self.variant(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if set.behavior.is_none() {
            self.r.note(K::MissingRequired, el, Some("cBhvr"));
        }
        Ok(set)
    }

    /// `p:CT_TLCommonMediaNodeData`.
    // spec-audit: owner=cMediaNode
    fn media(&mut self, el: &Elem, depth: usize) -> Read<CommonMediaNode> {
        let mut e = el.clone();
        let mut media = CommonMediaNode {
            vol: self.r.attr_in(&mut e, "vol", fixed_positive)?,
            mute: self.r.attr(&mut e, "mute")?,
            num_sld: self.r.attr(&mut e, "numSld")?,
            show_when_stopped: self.r.attr(&mut e, "showWhenStopped")?,
            ..CommonMediaNode::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "cTn" => once!(self, &child, media.common, self.common(&child, depth)?),
                "tgtEl" => once!(self, &child, media.target, self.target(&child)?),
                _ => self.r.retained(&child),
            }
        }
        if media.common.is_none() {
            self.r.note(K::MissingRequired, el, Some("cTn"));
        }
        if media.target.is_none() {
            self.r.note(K::MissingRequired, el, Some("tgtEl"));
        }
        Ok(media)
    }

    fn only_media(&mut self, el: &Elem, depth: usize) -> Read<Option<CommonMediaNode>> {
        let mut media = None;
        for child in self.r.children(el)? {
            if child.is(ns::PML, "cMediaNode") {
                once!(self, &child, media, self.media(&child, depth)?);
            } else {
                self.r.retained(&child);
            }
        }
        if media.is_none() {
            self.r.note(K::MissingRequired, el, Some("cMediaNode"));
        }
        Ok(media)
    }

    /// `p:CT_TLMediaNodeAudio`.
    // spec-audit: owner=audio
    fn audio(&mut self, el: &Elem, depth: usize) -> Read<AudioNode> {
        let mut e = el.clone();
        let is_narration = self.r.attr(&mut e, "isNarration")?;
        self.r.finish(&e);
        Ok(AudioNode {
            is_narration,
            media: self.only_media(el, depth)?,
        })
    }

    /// `p:CT_TLMediaNodeVideo`.
    // spec-audit: owner=video
    fn video(&mut self, el: &Elem, depth: usize) -> Read<VideoNode> {
        let mut e = el.clone();
        let full_scrn = self.r.attr(&mut e, "fullScrn")?;
        self.r.finish(&e);
        Ok(VideoNode {
            full_scrn,
            media: self.only_media(el, depth)?,
        })
    }

    /// `p:CT_BuildList`.
    // spec-audit: owner=bldLst
    fn builds(&mut self, el: &Elem) -> Read<Vec<BuildEntry>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.checked_children(el, self.limits.max_builds, "bldLst")? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "bldP" => out.push(BuildEntry::Paragraph(self.build_paragraph(&child)?)),
                "bldDgm" => out.push(BuildEntry::Diagram(self.build_diagram(&child)?)),
                "bldOleChart" => out.push(BuildEntry::OleChart(self.build_ole_chart(&child)?)),
                "bldGraphic" => out.push(BuildEntry::Graphic(self.build_graphic(&child)?)),
                _ => self.r.retained(&child),
            }
        }
        Ok(out)
    }

    /// `p:AG_TLBuild`.
    // spec-audit: owner=bldP,bldDgm,bldOleChart,bldGraphic
    fn build_target(&mut self, e: &mut Elem) -> Read<BuildTarget> {
        Ok(BuildTarget {
            spid: self.r.required(e, "spid", |_: &u32| true)?,
            grp_id: self.r.required(e, "grpId", |_: &u32| true)?,
            ui_expand: self.r.attr(e, "uiExpand")?,
        })
    }

    /// `p:CT_TLBuildParagraph`.
    // spec-audit: owner=bldP
    fn build_paragraph(&mut self, el: &Elem) -> Read<BuildParagraph> {
        let mut e = el.clone();
        let mut build = BuildParagraph {
            target: self.build_target(&mut e)?,
            build: self.r.attr(&mut e, "build")?,
            bld_lvl: self.r.attr(&mut e, "bldLvl")?,
            anim_bg: self.r.attr(&mut e, "animBg")?,
            auto_update_anim_bg: self.r.attr(&mut e, "autoUpdateAnimBg")?,
            rev: self.r.attr(&mut e, "rev")?,
            adv_auto: self.r.attr(&mut e, "advAuto")?,
            tmpl_lst: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if child.is(ns::PML, "tmplLst") {
                once!(self, &child, build.tmpl_lst, self.templates(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        Ok(build)
    }

    /// `p:CT_TLTemplateList`.
    // spec-audit: owner=tmplLst
    fn templates(&mut self, el: &Elem) -> Read<Vec<BuildTemplate>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.checked_children(el, self.limits.max_templates, "tmplLst")? {
            if child.is(ns::PML, "tmpl") {
                out.push(self.template(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        Ok(out)
    }

    /// `p:CT_TLTemplate`.
    // spec-audit: owner=tmpl
    fn template(&mut self, el: &Elem) -> Read<BuildTemplate> {
        let mut e = el.clone();
        let mut template = BuildTemplate {
            lvl: self.r.attr(&mut e, "lvl")?,
            tn_lst: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if child.is(ns::PML, "tnLst") {
                once!(self, &child, template.tn_lst, self.nodes_of(&child, 0)?);
            } else {
                self.r.retained(&child);
            }
        }
        if template.tn_lst.is_none() {
            self.r.note(K::MissingRequired, el, Some("tnLst"));
        }
        Ok(template)
    }

    /// `p:CT_TLBuildDiagram`.
    // spec-audit: owner=bldDgm
    fn build_diagram(&mut self, el: &Elem) -> Read<BuildDiagram> {
        let mut e = el.clone();
        let build = BuildDiagram {
            target: self.build_target(&mut e)?,
            bld: self.r.attr(&mut e, "bld")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(build)
    }

    /// `p:CT_TLOleBuildChart`.
    // spec-audit: owner=bldOleChart
    fn build_ole_chart(&mut self, el: &Elem) -> Read<BuildOleChart> {
        let mut e = el.clone();
        let build = BuildOleChart {
            target: self.build_target(&mut e)?,
            bld: self.r.attr(&mut e, "bld")?,
            anim_bg: self.r.attr(&mut e, "animBg")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(build)
    }

    /// `p:CT_TLGraphicalObjectBuild`.
    // spec-audit: owner=bldGraphic
    fn build_graphic(&mut self, el: &Elem) -> Read<BuildGraphic> {
        let mut e = el.clone();
        let mut build = BuildGraphic {
            target: self.build_target(&mut e)?,
            choice: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "bldAsOne" => {
                    self.empty(&child)?;
                    once!(self, &child, build.choice, GraphicBuildChoice::AsOne);
                }
                "bldSub" => {
                    let sub = self.build_sub(&child)?;
                    once!(self, &child, build.choice, GraphicBuildChoice::Sub(sub));
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(build)
    }

    /// `a:CT_AnimationGraphicalObjectBuildProperties`.
    // spec-audit: owner=bldSub
    fn build_sub(&mut self, el: &Elem) -> Read<GraphicalObjectBuildProperties> {
        self.r.finish(el);
        let mut sub = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "bldDgm" => {
                    let dgm = self.dgm_build(&child)?;
                    once!(
                        self,
                        &child,
                        sub,
                        GraphicalObjectBuildProperties::Diagram(dgm)
                    );
                }
                "bldChart" => {
                    let chart = self.chart_build(&child)?;
                    once!(
                        self,
                        &child,
                        sub,
                        GraphicalObjectBuildProperties::Chart(chart)
                    );
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(sub.unwrap_or(GraphicalObjectBuildProperties::Empty))
    }

    /// `a:CT_AnimationDgmBuildProperties`.
    // spec-audit: owner=bldDgm
    fn dgm_build(&mut self, el: &Elem) -> Read<DiagramBuildProperties> {
        let mut e = el.clone();
        let build = DiagramBuildProperties {
            bld: self.r.attr(&mut e, "bld")?,
            rev: self.r.attr(&mut e, "rev")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(build)
    }

    /// `a:CT_AnimationChartBuildProperties`.
    // spec-audit: owner=bldChart
    fn chart_build(&mut self, el: &Elem) -> Read<ChartBuildProperties> {
        let mut e = el.clone();
        let build = ChartBuildProperties {
            bld: self.r.attr(&mut e, "bld")?,
            anim_bg: self.r.attr(&mut e, "animBg")?,
        };
        self.r.finish(&e);
        self.nothing_inside(el)?;
        Ok(build)
    }
}

/// What the whole tree names, gathered in one walk.
#[derive(Default)]
struct Names {
    shapes: Vec<(&'static str, u32)>,
    node_ids: Vec<u32>,
    groups: Vec<u32>,
    references: Vec<u32>,
}

impl TargetVisitor for Names {
    fn shape(&mut self, element: &'static str, spid: &mut u32) {
        self.shapes.push((element, *spid));
    }
    fn common(&mut self, node: &mut CommonTimeNode) {
        self.node_ids.extend(node.node_id);
        self.groups.extend(node.grp_id);
    }
    fn node_reference(&mut self, val: u32) {
        self.references.push(val);
    }
}

/// Every `cNvPr/@id` under the part's `cSld`, in every branch, as a target may mean a
/// shape in a branch the resolver did not choose. Bounded by the tree, which is.
pub fn shape_ids(tree: &XmlTree) -> HashSet<u32> {
    let mut found = HashSet::new();
    let mut todo: Vec<ElementId> = tree.child_elements(tree.root());
    while let Some(id) = todo.pop() {
        if tree.local_name(id) == "cNvPr"
            && let Some(spid) = tree.plain_attribute(id, "id")
            && let Ok(spid) = spid.trim().parse()
        {
            found.insert(spid);
        }
        todo.extend(tree.child_elements(id));
    }
    found
}

/// Whether what the timing names is there. Reports and changes nothing, as a file may
/// hold what PowerPoint refuses, and it is kept.
pub(crate) fn check_references(r: &mut Reader<'_>, timing: &mut Slot<SlideTiming>) {
    let shapes = shape_ids(r.tree);
    for value in timing.values_mut() {
        let mut names = Names::default();
        value.visit(&mut names);
        for (element, spid) in &names.shapes {
            if !shapes.contains(spid) {
                r.findings.note(
                    K::Timing(TimingFinding::UnknownShape),
                    r.part,
                    element,
                    Some("spid"),
                );
            }
        }
        let mut ids = HashSet::new();
        for id in &names.node_ids {
            if !ids.insert(*id) {
                r.findings.note(K::DuplicateId, r.part, "cTn", Some("id"));
            }
        }
        if names.references.iter().any(|val| !ids.contains(val)) {
            r.findings
                .note(K::UnresolvedReference, r.part, "tn", Some("val"));
        }
        let mut built = HashSet::new();
        let mut groups = HashSet::new();
        for entry in value.bld_lst.iter().flatten() {
            let target = entry.target();
            if !built.insert((target.spid, target.grp_id)) {
                r.findings.note(
                    K::Timing(TimingFinding::BuildMismatch),
                    r.part,
                    "bldLst",
                    Some("grpId"),
                );
            }
            groups.extend(target.grp_id);
        }
        if value.bld_lst.is_some() && names.groups.iter().any(|g| !groups.contains(g)) {
            r.findings.note(
                K::Timing(TimingFinding::BuildMismatch),
                r.part,
                "cTn",
                Some("grpId"),
            );
        }
    }
}

/// The attribute a 2010 namespace names, as written on this element.
pub(crate) fn p14_name(el: &Elem, local: &str) -> Option<String> {
    el.name_of_ns(P14, local).map(str::to_owned)
}
