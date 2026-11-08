//! The Motion family (`docs/24`, `VS-10`): a slide's transition, and the effects of its
//! main animation sequence, as PowerPoint's Transitions and Animations tabs edit them.
//!
//! Each operation returns its exact inverse, as every operation here does. For the
//! effects that inverse is [`Operation::SetTiming`] with the timing as it was, so an
//! undo gives back the very tree that was read, ids and all, and the writer, which
//! patches only what differs (ADR-013), then changes nothing.
//!
//! # The main sequence
//!
//! PowerPoint writes one shape for it, and this reads and writes that shape:
//!
//! ```text
//! timing/tnLst/par[tmRoot]/childTnLst/seq[mainSeq]/childTnLst
//!   par            one per click; the first may start by itself (onBegin)
//!     par          one per "after previous" step, delayed by the steps before it
//!       par        one per effect: presetClass, presetID, nodeType, its behaviours
//! ```
//!
//! An effect from a file keeps its own behaviours whole; a move or a delete only
//! regroups it. A timing in any other shape (an `mc:AlternateContent`, one left
//! unread, a root holding something else) is refused with `CS-TXN-0008` and changes
//! nothing, so an edit never loses what it cannot read. Every `cTn/@id` is numbered
//! again in file order after an edit, as PowerPoint does on save, and `tn/@val`
//! references follow their node.

use std::collections::BTreeMap;

use casual_slide_model::presentation::Percentage;
use casual_slide_model::slide::{SlideKind, SlidePart, SlideParts};
use casual_slide_model::timing::{
    AnimVariant, AnimateBehavior, AnimateCalcMode, AnimateEffectBehavior, AnimateEffectTransition,
    AnimateScaleBehavior, AnimateValueTime, AnimateValueType, BehaviorAdditive, BuildEntry,
    BuildParagraph, BuildTarget, CommonBehavior, CommonTimeNode, ConditionTrigger,
    ContainerTimeNode, NextAction, SequenceTimeNode, SetBehavior, ShapeTarget, SlideTiming, Slot,
    TargetVisitor, TimeAnimateValue, TimeCondition, TimeNode, TimeNodeFill, TimeNodePresetClass,
    TimeNodeRestart, TimeNodeType, TimePoint, TimeTarget, TimeValue, TriggerEvent,
};
use casual_slide_model::transition::{
    InOutDirection, OptionalBlackTransition, OrientationDirection, SideDirection,
    SideDirectionTransition, SlideTransition, SplitTransition, TransitionEffect, TransitionSpeed,
};

use super::{Applied, Operation, TxnError};

/// The longest duration an effect or a transition is given, in milliseconds: a
/// minute, PowerPoint's own limit for a transition.
pub const MAX_DURATION_MS: u32 = 60_000;

/// The transitions the Transitions tab offers (`VS-10`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    /// None: the slide changes at once, with no `p:transition` effect.
    None,
    /// `p:cut`.
    Cut,
    /// `p:fade`.
    Fade,
    /// `p:push`, from the right.
    Push,
    /// `p:wipe`, from the right.
    Wipe,
    /// `p:split`, horizontal, out.
    Split,
}

impl TransitionKind {
    /// From the bridge's number: 0 none, 1 cut, 2 fade, 3 push, 4 wipe, 5 split.
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::None,
            1 => Self::Cut,
            2 => Self::Fade,
            3 => Self::Push,
            4 => Self::Wipe,
            5 => Self::Split,
            _ => return None,
        })
    }

    /// The bridge's number.
    pub fn code(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Cut => 1,
            Self::Fade => 2,
            Self::Push => 3,
            Self::Wipe => 4,
            Self::Split => 5,
        }
    }

    /// PowerPoint's default duration for it, in milliseconds.
    pub fn default_ms(self) -> u32 {
        match self {
            Self::None | Self::Cut => 0,
            Self::Fade => 700,
            Self::Push | Self::Wipe => 1000,
            Self::Split => 1500,
        }
    }

    /// Which of these a transition is, if one. `None` for an effect the tab does not
    /// offer, which it shows as no choice and leaves alone until one is made.
    pub fn of(t: &SlideTransition) -> Option<Self> {
        Some(match &t.effect {
            None => Self::None,
            Some(TransitionEffect::Cut(_)) => Self::Cut,
            Some(TransitionEffect::Fade(_)) => Self::Fade,
            Some(TransitionEffect::Push(_)) => Self::Push,
            Some(TransitionEffect::Wipe(_)) => Self::Wipe,
            Some(TransitionEffect::Split(_)) => Self::Split,
            Some(_) => return None,
        })
    }

    fn effect(self) -> Option<TransitionEffect> {
        Some(match self {
            Self::None => return None,
            Self::Cut => TransitionEffect::Cut(OptionalBlackTransition::default()),
            Self::Fade => TransitionEffect::Fade(OptionalBlackTransition::default()),
            Self::Push => TransitionEffect::Push(SideDirectionTransition {
                dir: Some(SideDirection::Left),
            }),
            Self::Wipe => TransitionEffect::Wipe(SideDirectionTransition {
                dir: Some(SideDirection::Left),
            }),
            Self::Split => TransitionEffect::Split(SplitTransition {
                orient: Some(OrientationDirection::Horizontal),
                dir: Some(InOutDirection::Out),
            }),
        })
    }
}

/// `@spd` for a duration, for a reader that does not know `p14:dur`.
fn speed_for(ms: u32) -> TransitionSpeed {
    match ms {
        0..=500 => TransitionSpeed::Fast,
        501..=750 => TransitionSpeed::Medium,
        _ => TransitionSpeed::Slow,
    }
}

/// The transition the Transitions tab writes: `kind` over `duration_ms`, keeping what
/// `existing` says about advancing (`@advClick`, `@advTm`) and its sound.
///
/// As PowerPoint writes it: the duration is `p14:dur`, which only a branch requiring
/// `p14` may hold, so an effect comes as `mc:AlternateContent` with that branch and a
/// fallback giving `@spd` alone. `None` with nothing to keep is no transition at all.
pub fn build_transition(
    kind: TransitionKind,
    duration_ms: u32,
    existing: Option<&Slot<SlideTransition>>,
) -> Option<Slot<SlideTransition>> {
    let kept = existing.and_then(Slot::resolved);
    let base = SlideTransition {
        adv_click: kept.and_then(|t| t.adv_click),
        adv_tm: kept.and_then(|t| t.adv_tm),
        snd_ac: kept.and_then(|t| t.snd_ac.clone()),
        ..SlideTransition::default()
    };
    let Some(effect) = kind.effect() else {
        return (base != SlideTransition::default()).then(|| Slot::Plain(Box::new(base)));
    };
    let ms = duration_ms.min(MAX_DURATION_MS);
    if kind == TransitionKind::Cut {
        return Some(Slot::Plain(Box::new(SlideTransition {
            effect: Some(effect),
            ..base
        })));
    }
    let fallback = SlideTransition {
        spd: Some(speed_for(ms)),
        effect: Some(effect),
        ..base
    };
    let choice = SlideTransition {
        p14_dur: Some(ms.to_string()),
        ..fallback.clone()
    };
    Some(Slot::Alternate(casual_slide_model::timing::Alternates {
        branches: vec![
            casual_slide_model::timing::AlternateBranch {
                requires: Some("p14".to_owned()),
                is_choice: true,
                value: Some(Box::new(choice)),
            },
            casual_slide_model::timing::AlternateBranch {
                requires: None,
                is_choice: false,
                value: Some(Box::new(fallback)),
            },
        ],
        // As the reader resolves it: `p14` is not a namespace it understands.
        selected: Some(1),
    }))
}

/// The effects the Animations tab offers (`VS-10`), with PowerPoint's preset ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    /// Entrance: Appear (preset 1).
    Appear,
    /// Entrance: Fade (preset 10).
    Fade,
    /// Entrance: Fly In from the bottom (preset 2, subtype 4).
    FlyIn,
    /// Emphasis: Pulse (preset 26).
    Pulse,
    /// Exit: Disappear (preset 1).
    Disappear,
    /// Exit: Fade (preset 10), "Fade Out" in the gallery.
    FadeOut,
}

impl EffectKind {
    /// From the bridge's number: 0 Appear, 1 Fade, 2 Fly In, 3 Pulse, 4 Disappear,
    /// 5 Fade Out.
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::Appear,
            1 => Self::Fade,
            2 => Self::FlyIn,
            3 => Self::Pulse,
            4 => Self::Disappear,
            5 => Self::FadeOut,
            _ => return None,
        })
    }

    /// The bridge's number.
    pub fn code(self) -> u32 {
        match self {
            Self::Appear => 0,
            Self::Fade => 1,
            Self::FlyIn => 2,
            Self::Pulse => 3,
            Self::Disappear => 4,
            Self::FadeOut => 5,
        }
    }

    /// `@presetClass`.
    pub fn class(self) -> TimeNodePresetClass {
        match self {
            Self::Appear | Self::Fade | Self::FlyIn => TimeNodePresetClass::Entrance,
            Self::Pulse => TimeNodePresetClass::Emphasis,
            Self::Disappear | Self::FadeOut => TimeNodePresetClass::Exit,
        }
    }

    fn preset(self) -> (i32, i32) {
        match self {
            Self::Appear | Self::Disappear => (1, 0),
            Self::Fade | Self::FadeOut => (10, 0),
            Self::FlyIn => (2, 4),
            Self::Pulse => (26, 0),
        }
    }

    /// PowerPoint's default duration, in milliseconds. Appear and Disappear are
    /// instant.
    pub fn default_ms(self) -> u32 {
        match self {
            Self::Appear | Self::Disappear => 0,
            Self::Fade | Self::FlyIn | Self::FadeOut => 500,
            Self::Pulse => 500,
        }
    }

    /// Which of these a preset is, by class and id.
    fn of(class: Option<TimeNodePresetClass>, id: Option<i32>) -> Option<Self> {
        use TimeNodePresetClass as C;
        Some(match (class?, id?) {
            (C::Entrance, 1) => Self::Appear,
            (C::Entrance, 10) => Self::Fade,
            (C::Entrance, 2) => Self::FlyIn,
            (C::Emphasis, 26) => Self::Pulse,
            (C::Exit, 1) => Self::Disappear,
            (C::Exit, 10) => Self::FadeOut,
            _ => return None,
        })
    }
}

/// When an effect starts, as the Animations tab's Start box says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectStart {
    /// On Click: `clickEffect`.
    OnClick,
    /// With Previous: `withEffect`.
    WithPrevious,
    /// After Previous: `afterEffect`.
    AfterPrevious,
}

impl EffectStart {
    /// From the bridge's number: 0 on click, 1 with previous, 2 after previous.
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::OnClick,
            1 => Self::WithPrevious,
            2 => Self::AfterPrevious,
            _ => return None,
        })
    }

    /// The bridge's number.
    pub fn code(self) -> u32 {
        match self {
            Self::OnClick => 0,
            Self::WithPrevious => 1,
            Self::AfterPrevious => 2,
        }
    }

    fn node_type(self) -> TimeNodeType {
        match self {
            Self::OnClick => TimeNodeType::ClickEffect,
            Self::WithPrevious => TimeNodeType::WithEffect,
            Self::AfterPrevious => TimeNodeType::AfterEffect,
        }
    }
}

/// An effect to add.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewEffect {
    /// The shape, by its `cNvPr/@id`; a shape on the slide must have it.
    pub spid: u32,
    /// Which effect.
    pub kind: EffectKind,
    /// When it starts.
    pub start: EffectStart,
    /// How long it runs, in milliseconds; held to [`MAX_DURATION_MS`]. Ignored for
    /// Appear and Disappear, which are instant.
    pub duration_ms: u32,
}

/// One effect of a slide's main sequence, as the Animation Pane lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectEntry {
    /// The shape its first target names.
    pub spid: Option<u32>,
    /// Which of the tab's effects it is; `None` for any other preset, which still
    /// plays in PowerPoint and is kept whole.
    pub kind: Option<EffectKind>,
    /// `@presetClass`.
    pub class: Option<TimeNodePresetClass>,
    /// When it starts.
    pub start: EffectStart,
    /// How long it runs, in milliseconds, from its behaviours.
    pub duration_ms: u32,
}

/* ── Building nodes ──────────────────────────────────────────────────────── */

fn delay(ms: TimeValue) -> TimeCondition {
    TimeCondition {
        delay: Some(ms),
        ..TimeCondition::default()
    }
}

fn starts_at(ms: u32) -> Option<Vec<TimeCondition>> {
    Some(vec![delay(TimeValue::Milliseconds(ms))])
}

fn shape_target(spid: u32) -> Option<TimeTarget> {
    Some(TimeTarget::Shape(ShapeTarget {
        spid: Some(spid),
        sub: None,
    }))
}

fn behavior(spid: u32, common: CommonTimeNode, attrs: &[&str]) -> Option<CommonBehavior> {
    Some(CommonBehavior {
        common: Some(common),
        target: shape_target(spid),
        attr_name_lst: (!attrs.is_empty()).then(|| attrs.iter().map(|a| (*a).to_owned()).collect()),
        ..CommonBehavior::default()
    })
}

fn set_visibility(spid: u32, at: u32, visible: bool) -> TimeNode {
    TimeNode::Set(SetBehavior {
        behavior: behavior(
            spid,
            CommonTimeNode {
                dur: Some(TimeValue::Milliseconds(1)),
                fill: Some(TimeNodeFill::Hold),
                st_cond_lst: starts_at(at),
                ..CommonTimeNode::default()
            },
            &["style.visibility"],
        ),
        to: Some(AnimVariant::Str(Some(
            if visible { "visible" } else { "hidden" }.to_owned(),
        ))),
    })
}

fn fade(spid: u32, ms: u32, transition: AnimateEffectTransition) -> TimeNode {
    TimeNode::AnimEffect(AnimateEffectBehavior {
        transition: Some(transition),
        filter: Some("fade".to_owned()),
        behavior: behavior(
            spid,
            CommonTimeNode {
                dur: Some(TimeValue::Milliseconds(ms)),
                ..CommonTimeNode::default()
            },
            &[],
        ),
        ..AnimateEffectBehavior::default()
    })
}

fn tav(at: i32, value: &str) -> TimeAnimateValue {
    TimeAnimateValue {
        tm: Some(AnimateValueTime::At(Percentage(at))),
        fmla: None,
        val: Some(AnimVariant::Str(Some(value.to_owned()))),
    }
}

fn fly(spid: u32, ms: u32, attr: &str, from: &str, to: &str) -> TimeNode {
    TimeNode::Anim(AnimateBehavior {
        calcmode: Some(AnimateCalcMode::Linear),
        value_type: Some(AnimateValueType::Number),
        behavior: Some(CommonBehavior {
            additive: Some(BehaviorAdditive::Base),
            ..behavior(
                spid,
                CommonTimeNode {
                    dur: Some(TimeValue::Milliseconds(ms)),
                    fill: Some(TimeNodeFill::Hold),
                    ..CommonTimeNode::default()
                },
                &[attr],
            )
            .unwrap_or_default()
        }),
        tav_lst: Some(vec![tav(0, from), tav(100_000, to)]),
        ..AnimateBehavior::default()
    })
}

fn pulse(spid: u32, ms: u32) -> TimeNode {
    TimeNode::AnimScale(AnimateScaleBehavior {
        behavior: behavior(
            spid,
            CommonTimeNode {
                dur: Some(TimeValue::Milliseconds((ms / 2).max(1))),
                auto_rev: Some(true),
                fill: Some(TimeNodeFill::Hold),
                ..CommonTimeNode::default()
            },
            &[],
        ),
        by: Some(TimePoint {
            x: Some(Percentage(105_000)),
            y: Some(Percentage(105_000)),
        }),
        ..AnimateScaleBehavior::default()
    })
}

/// The effect's own `par`, as PowerPoint writes the preset.
fn effect_node(e: &NewEffect) -> TimeNode {
    let ms = e.duration_ms.clamp(1, MAX_DURATION_MS);
    let spid = e.spid;
    let children = match e.kind {
        EffectKind::Appear => vec![set_visibility(spid, 0, true)],
        EffectKind::Fade => vec![
            set_visibility(spid, 0, true),
            fade(spid, ms, AnimateEffectTransition::In),
        ],
        EffectKind::FlyIn => vec![
            set_visibility(spid, 0, true),
            fly(spid, ms, "ppt_x", "#ppt_x", "#ppt_x"),
            fly(spid, ms, "ppt_y", "1+#ppt_h/2", "#ppt_y"),
        ],
        EffectKind::Pulse => vec![pulse(spid, ms)],
        EffectKind::Disappear => vec![set_visibility(spid, 0, false)],
        EffectKind::FadeOut => vec![
            fade(spid, ms, AnimateEffectTransition::Out),
            set_visibility(spid, ms.saturating_sub(1), false),
        ],
    };
    let (id, subtype) = e.kind.preset();
    TimeNode::Par(ContainerTimeNode {
        common: Some(CommonTimeNode {
            preset_id: Some(id),
            preset_class: Some(e.kind.class()),
            preset_subtype: Some(subtype),
            fill: Some(TimeNodeFill::Hold),
            grp_id: Some(0),
            node_type: Some(e.start.node_type()),
            st_cond_lst: starts_at(0),
            child_tn_lst: Some(children),
            ..CommonTimeNode::default()
        }),
    })
}

fn main_sequence(groups: Vec<TimeNode>) -> TimeNode {
    let slide_event = |evt| TimeCondition {
        evt: Some(evt),
        delay: Some(TimeValue::Milliseconds(0)),
        trigger: Some(ConditionTrigger::Target(TimeTarget::Slide)),
    };
    TimeNode::Seq(SequenceTimeNode {
        concurrent: Some(true),
        next_ac: Some(NextAction::Seek),
        common: Some(CommonTimeNode {
            dur: Some(TimeValue::Indefinite),
            node_type: Some(TimeNodeType::MainSequence),
            child_tn_lst: Some(groups),
            ..CommonTimeNode::default()
        }),
        prev_cond_lst: Some(vec![slide_event(TriggerEvent::OnPrevious)]),
        next_cond_lst: Some(vec![slide_event(TriggerEvent::OnNext)]),
        prev_ac: None,
    })
}

fn root(children: Vec<TimeNode>) -> TimeNode {
    TimeNode::Par(ContainerTimeNode {
        common: Some(CommonTimeNode {
            dur: Some(TimeValue::Indefinite),
            restart: Some(TimeNodeRestart::Never),
            node_type: Some(TimeNodeType::Root),
            child_tn_lst: Some(children),
            ..CommonTimeNode::default()
        }),
    })
}

/* ── Reading ─────────────────────────────────────────────────────────────── */

fn common_of(node: &TimeNode) -> Option<&CommonTimeNode> {
    match node {
        TimeNode::Par(n) | TimeNode::Excl(n) => n.common.as_ref(),
        TimeNode::Seq(n) => n.common.as_ref(),
        TimeNode::Anim(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::AnimClr(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::AnimEffect(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::AnimMotion(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::AnimRot(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::AnimScale(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::Cmd(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::Set(n) => n.behavior.as_ref()?.common.as_ref(),
        TimeNode::Audio(n) => n.media.as_ref()?.common.as_ref(),
        TimeNode::Video(n) => n.media.as_ref()?.common.as_ref(),
        TimeNode::Alternate(_) => None,
    }
}

fn ms_of(value: Option<TimeValue>) -> u64 {
    match value {
        Some(TimeValue::Milliseconds(ms)) => u64::from(ms),
        _ => 0,
    }
}

/// How long a node runs from its own start, in milliseconds: its delay, then the
/// longer of its own duration (twice, reversing) and its children's. A tree is bounded
/// by the reader's depth limit, so this recursion is too.
fn length(node: &TimeNode) -> u64 {
    let Some(c) = common_of(node) else { return 0 };
    let start = c
        .st_cond_lst
        .as_ref()
        .and_then(|l| l.first())
        .map_or(0, |cond| ms_of(cond.delay));
    let own = ms_of(c.dur).saturating_mul(if c.auto_rev == Some(true) { 2 } else { 1 });
    let kids = c
        .child_tn_lst
        .iter()
        .flatten()
        .map(length)
        .max()
        .unwrap_or(0);
    start.saturating_add(own.max(kids))
}

struct FirstSpid(Option<u32>);

impl TargetVisitor for FirstSpid {
    fn shape(&mut self, _element: &'static str, spid: &mut u32) {
        self.0.get_or_insert(*spid);
    }
}

fn entry_of(start: EffectStart, node: &TimeNode) -> EffectEntry {
    let c = common_of(node);
    let class = c.and_then(|c| c.preset_class);
    let mut spid = FirstSpid(None);
    node.clone().visit(&mut spid);
    let ms = length(node).saturating_sub(
        c.and_then(|c| c.st_cond_lst.as_ref()?.first().map(|d| ms_of(d.delay)))
            .unwrap_or(0),
    );
    EffectEntry {
        spid: spid.0,
        kind: EffectKind::of(class, c.and_then(|c| c.preset_id)),
        class,
        start,
        duration_ms: u32::try_from(ms).unwrap_or(u32::MAX),
    }
}

fn children(node: &TimeNode) -> Option<&Vec<TimeNode>> {
    match node {
        TimeNode::Par(n) => n.common.as_ref()?.child_tn_lst.as_ref(),
        _ => None,
    }
}

fn is_type(node: &TimeNode, kind: TimeNodeType) -> bool {
    common_of(node).and_then(|c| c.node_type) == Some(kind)
}

/// A slide's timing taken apart: the root's other children and where the main
/// sequence was among them, the effects in order, and the build list.
struct Parts {
    others: Vec<TimeNode>,
    at: usize,
    effects: Vec<(EffectStart, TimeNode)>,
    builds: Vec<BuildEntry>,
    /// The main sequence's own `cTn` and conditions, kept as read.
    seq: Option<SequenceTimeNode>,
    root_common: Option<CommonTimeNode>,
}

fn take_apart(ordinal: u32, timing: Option<&Slot<SlideTiming>>) -> Result<Parts, TxnError> {
    let refuse = TxnError::Timing(ordinal);
    let timing = match timing {
        None => {
            return Ok(Parts {
                others: Vec::new(),
                at: 0,
                effects: Vec::new(),
                builds: Vec::new(),
                seq: None,
                root_common: None,
            });
        }
        Some(Slot::Plain(t)) => t,
        Some(_) => return Err(refuse),
    };
    let builds = timing.bld_lst.clone().unwrap_or_default();
    let (root_node, root_common) = match timing.tn_lst.as_deref() {
        None | Some([]) => (None, None),
        Some([node @ TimeNode::Par(p)]) if is_type(node, TimeNodeType::Root) => {
            (Some(node), p.common.clone())
        }
        Some(_) => return Err(refuse),
    };
    let mut others = root_node.and_then(children).cloned().unwrap_or_default();
    let found = others
        .iter()
        .position(|n| matches!(n, TimeNode::Seq(_)) && is_type(n, TimeNodeType::MainSequence));
    let Some(at) = found else {
        return Ok(Parts {
            others,
            at: 0,
            effects: Vec::new(),
            builds,
            seq: None,
            root_common,
        });
    };
    let TimeNode::Seq(seq) = others.remove(at) else {
        return Err(refuse);
    };
    let mut effects = Vec::new();
    let clicks = seq
        .common
        .as_ref()
        .and_then(|c| c.child_tn_lst.as_ref())
        .cloned()
        .unwrap_or_default();
    for click in &clicks {
        let groups = children(click).ok_or(refuse.clone())?;
        for (g, group) in groups.iter().enumerate() {
            let items = children(group).ok_or(refuse.clone())?;
            for (i, item) in items.iter().enumerate() {
                if !matches!(item, TimeNode::Par(_)) {
                    return Err(refuse);
                }
                let start = match common_of(item).and_then(|c| c.node_type) {
                    Some(TimeNodeType::ClickEffect) => EffectStart::OnClick,
                    Some(TimeNodeType::WithEffect) => EffectStart::WithPrevious,
                    Some(TimeNodeType::AfterEffect) => EffectStart::AfterPrevious,
                    _ if i > 0 => EffectStart::WithPrevious,
                    _ if g > 0 => EffectStart::AfterPrevious,
                    _ => EffectStart::OnClick,
                };
                effects.push((start, item.clone()));
            }
        }
    }
    Ok(Parts {
        others,
        at,
        effects,
        builds,
        seq: Some(seq),
        root_common,
    })
}

/// The effects of a slide's main sequence, in order, for the Animation Pane. A
/// timing in a shape this does not edit gives `CS-TXN-0008`, with `ordinal` named.
pub fn effects(
    ordinal: u32,
    timing: Option<&Slot<SlideTiming>>,
) -> Result<Vec<EffectEntry>, TxnError> {
    Ok(take_apart(ordinal, timing)?
        .effects
        .iter()
        .map(|(start, node)| entry_of(*start, node))
        .collect())
}

/* ── Writing ─────────────────────────────────────────────────────────────── */

fn group(fill_children: Vec<TimeNode>, conditions: Vec<TimeCondition>) -> TimeNode {
    TimeNode::Par(ContainerTimeNode {
        common: Some(CommonTimeNode {
            fill: Some(TimeNodeFill::Hold),
            st_cond_lst: Some(conditions),
            child_tn_lst: Some(fill_children),
            ..CommonTimeNode::default()
        }),
    })
}

fn set_node_type(node: &mut TimeNode, start: EffectStart) {
    if let TimeNode::Par(p) = node
        && let Some(c) = p.common.as_mut()
    {
        c.node_type = Some(start.node_type());
    }
}

/// A sentinel id for the main sequence while the tree is rebuilt, before every id
/// is numbered again.
const MAIN_SEQ_MARK: u32 = u32::MAX;

/// The main sequence's click groups from its effects in order.
fn regroup(effects: Vec<(EffectStart, TimeNode)>, main_id: u32) -> Vec<TimeNode> {
    let auto_first = effects
        .first()
        .is_some_and(|(s, _)| *s != EffectStart::OnClick);
    // Clicks, each a list of steps, each a list of effects.
    let mut clicks: Vec<Vec<Vec<TimeNode>>> = Vec::new();
    for (start, mut node) in effects {
        set_node_type(&mut node, start);
        match (start, clicks.last_mut()) {
            (EffectStart::WithPrevious, Some(steps)) => match steps.last_mut() {
                Some(step) => step.push(node),
                None => steps.push(vec![node]),
            },
            (EffectStart::AfterPrevious, Some(steps)) => steps.push(vec![node]),
            _ => clicks.push(vec![vec![node]]),
        }
    }
    clicks
        .into_iter()
        .enumerate()
        .map(|(c, steps)| {
            let mut at: u64 = 0;
            let mut out = Vec::with_capacity(steps.len());
            for step in steps {
                let longest = step.iter().map(length).max().unwrap_or(0);
                let start = u32::try_from(at).unwrap_or(u32::MAX);
                out.push(group(step, vec![delay(TimeValue::Milliseconds(start))]));
                at = at.saturating_add(longest);
            }
            let mut conditions = vec![delay(TimeValue::Indefinite)];
            if c == 0 && auto_first {
                conditions.push(TimeCondition {
                    evt: Some(TriggerEvent::OnBegin),
                    delay: Some(TimeValue::Milliseconds(0)),
                    trigger: Some(ConditionTrigger::Node(Some(main_id))),
                });
            }
            group(out, conditions)
        })
        .collect()
}

/// Number every `cTn` 1, 2, 3… in file order; references follow their node.
struct Renumber {
    next: u32,
    map: BTreeMap<u32, u32>,
}

impl TargetVisitor for Renumber {
    fn shape(&mut self, _element: &'static str, _spid: &mut u32) {}
    fn common(&mut self, node: &mut CommonTimeNode) {
        self.next = self.next.saturating_add(1);
        if let Some(old) = node.node_id {
            self.map.entry(old).or_insert(self.next);
        }
        node.node_id = Some(self.next);
    }
}

struct Follow<'a>(&'a BTreeMap<u32, u32>);

impl Follow<'_> {
    fn fix(&self, list: &mut Option<Vec<TimeCondition>>) {
        for cond in list.iter_mut().flatten() {
            if let Some(ConditionTrigger::Node(Some(val))) = &mut cond.trigger
                && let Some(new) = self.0.get(val)
            {
                *val = *new;
            }
        }
    }
}

impl TargetVisitor for Follow<'_> {
    fn shape(&mut self, _element: &'static str, _spid: &mut u32) {}
    fn common(&mut self, node: &mut CommonTimeNode) {
        self.fix(&mut node.st_cond_lst);
        self.fix(&mut node.end_cond_lst);
        if let Some(cond) = node.end_sync.as_mut() {
            let mut one = Some(vec![cond.clone()]);
            self.fix(&mut one);
            if let Some(c) = one.and_then(|v| v.into_iter().next()) {
                *cond = c;
            }
        }
    }
}

fn renumber(timing: &mut SlideTiming) {
    let mut ids = Renumber {
        next: 0,
        map: BTreeMap::new(),
    };
    timing.visit(&mut ids);
    timing.visit(&mut Follow(&ids.map));
}

fn put_together(mut parts: Parts) -> Option<Slot<SlideTiming>> {
    if !parts.effects.is_empty() {
        let mut seq = parts
            .seq
            .take()
            .unwrap_or_else(|| match main_sequence(Vec::new()) {
                TimeNode::Seq(s) => s,
                _ => SequenceTimeNode::default(),
            });
        let common = seq.common.get_or_insert_with(CommonTimeNode::default);
        let main_id = *common.node_id.get_or_insert(MAIN_SEQ_MARK);
        common.child_tn_lst = Some(regroup(parts.effects, main_id));
        let at = parts.at.min(parts.others.len());
        parts.others.insert(at, TimeNode::Seq(seq));
    }
    if parts.others.is_empty() {
        return None;
    }
    let tn = match parts.root_common {
        Some(mut c) => {
            c.child_tn_lst = Some(parts.others);
            TimeNode::Par(ContainerTimeNode { common: Some(c) })
        }
        None => root(parts.others),
    };
    let mut timing = SlideTiming {
        tn_lst: Some(vec![tn]),
        bld_lst: (!parts.builds.is_empty()).then_some(parts.builds),
    };
    renumber(&mut timing);
    Some(Slot::Plain(Box::new(timing)))
}

fn slide_mut(parts: &mut SlideParts, ordinal: u32) -> Result<&mut SlidePart, TxnError> {
    let is_slide = parts
        .structure
        .entry(ordinal)
        .is_some_and(|e| e.kind == SlideKind::Slide);
    parts
        .get_mut(ordinal)
        .filter(|_| is_slide)
        .ok_or(TxnError::BadPart(ordinal))
}

/// Edit a slide's main sequence through `edit`, and put the timing back together.
fn edit_effects(
    parts: &mut SlideParts,
    ordinal: u32,
    edit: impl FnOnce(&SlidePart, &mut Parts) -> Result<(), TxnError>,
) -> Result<Applied, TxnError> {
    let part = slide_mut(parts, ordinal)?;
    let mut apart = take_apart(ordinal, part.timing.as_ref())?;
    edit(part, &mut apart)?;
    let after = put_together(apart);
    let before = core::mem::replace(&mut part.timing, after);
    let changed = before != part.timing;
    Ok((
        Operation::SetTiming {
            ordinal,
            timing: before,
        },
        changed,
    ))
}

/// Whether any effect still names `spid`.
fn still_animated(effects: &[(EffectStart, TimeNode)], others: &[TimeNode], spid: u32) -> bool {
    let mut named = Vec::new();
    struct All<'a>(&'a mut Vec<u32>);
    impl TargetVisitor for All<'_> {
        fn shape(&mut self, _element: &'static str, spid: &mut u32) {
            self.0.push(*spid);
        }
    }
    for node in effects.iter().map(|(_, n)| n).chain(others) {
        node.clone().visit(&mut All(&mut named));
    }
    named.contains(&spid)
}

pub(crate) fn add_effect(
    parts: &mut SlideParts,
    ordinal: u32,
    index: usize,
    effect: NewEffect,
) -> Result<Applied, TxnError> {
    edit_effects(parts, ordinal, |part, apart| {
        if index > apart.effects.len() {
            return Err(TxnError::OutOfRange {
                offset: index,
                length: apart.effects.len(),
            });
        }
        let node = part
            .animated_node(effect.spid)
            .and_then(|id| part.node(id))
            .ok_or(TxnError::NoTarget(effect.spid))?;
        // PowerPoint lists a shape with text in the build list, its text animated
        // with it as one (`animBg`).
        let listed = apart
            .builds
            .iter()
            .any(|b| b.target().spid == Some(effect.spid));
        if node.text_body.is_some() && !listed {
            apart.builds.push(BuildEntry::Paragraph(BuildParagraph {
                target: BuildTarget {
                    spid: Some(effect.spid),
                    grp_id: Some(0),
                    ui_expand: Some(true),
                },
                anim_bg: Some(true),
                ..BuildParagraph::default()
            }));
        }
        apart
            .effects
            .insert(index, (effect.start, effect_node(&effect)));
        Ok(())
    })
}

pub(crate) fn remove_effect(
    parts: &mut SlideParts,
    ordinal: u32,
    index: usize,
) -> Result<Applied, TxnError> {
    edit_effects(parts, ordinal, |_, apart| {
        if index >= apart.effects.len() {
            return Err(TxnError::OutOfRange {
                offset: index,
                length: apart.effects.len(),
            });
        }
        let (_, node) = apart.effects.remove(index);
        let mut named = FirstSpid(None);
        node.clone().visit(&mut named);
        if let Some(spid) = named.0
            && !still_animated(&apart.effects, &apart.others, spid)
        {
            apart.builds.retain(|b| {
                !(matches!(b, BuildEntry::Paragraph(_)) && b.target().spid == Some(spid))
            });
        }
        Ok(())
    })
}

pub(crate) fn move_effect(
    parts: &mut SlideParts,
    ordinal: u32,
    from: usize,
    to: usize,
) -> Result<Applied, TxnError> {
    edit_effects(parts, ordinal, |_, apart| {
        let len = apart.effects.len();
        for at in [from, to] {
            if at >= len {
                return Err(TxnError::OutOfRange {
                    offset: at,
                    length: len,
                });
            }
        }
        let item = apart.effects.remove(from);
        apart.effects.insert(to, item);
        Ok(())
    })
}

pub(crate) fn set_effect_start(
    parts: &mut SlideParts,
    ordinal: u32,
    index: usize,
    start: EffectStart,
) -> Result<Applied, TxnError> {
    edit_effects(parts, ordinal, |_, apart| {
        let len = apart.effects.len();
        let item = apart.effects.get_mut(index).ok_or(TxnError::OutOfRange {
            offset: index,
            length: len,
        })?;
        item.0 = start;
        Ok(())
    })
}

pub(crate) fn set_timing(
    parts: &mut SlideParts,
    ordinal: u32,
    timing: Option<Slot<SlideTiming>>,
) -> Result<Applied, TxnError> {
    let part = slide_mut(parts, ordinal)?;
    let changed = part.timing != timing;
    let before = core::mem::replace(&mut part.timing, timing);
    Ok((
        Operation::SetTiming {
            ordinal,
            timing: before,
        },
        changed,
    ))
}

pub(crate) fn set_transition(
    parts: &mut SlideParts,
    ordinal: u32,
    transition: Option<Slot<SlideTransition>>,
) -> Result<Applied, TxnError> {
    let part = slide_mut(parts, ordinal)?;
    let changed = part.transition != transition;
    let before = core::mem::replace(&mut part.transition, transition);
    Ok((
        Operation::SetTransition {
            ordinal,
            transition: before,
        },
        changed,
    ))
}
