// The Motion family's tests (`VS-10`): each op and its inverse, the main sequence's
// shape, and what is refused.

use super::animate::{EffectEntry, build_transition, effects};
use super::*;
use casual_slide_model::shape::{NodeKind, NonVisualDrawing, ShapeNode};
use casual_slide_model::slide::{CommonSlideData, DeckStructure, SlideKind, SlidePart};
use casual_slide_model::text::TextBody;
use casual_slide_model::timing::{
    BuildEntry, ConditionTrigger, TimeNode, TimeNodePresetClass, TimeNodeType, TimeValue,
};

const SLIDE: u32 = 0;

fn shape(node: u32, spid: u32, text: bool) -> ShapeNode {
    let mut s = ShapeNode::new(ObjectId { part: SLIDE, node }, NodeKind::Shape);
    s.non_visual = Some(NonVisualDrawing {
        spid: Some(spid),
        ..NonVisualDrawing::default()
    });
    if text {
        s.text_body = Some(Box::new(TextBody {
            paragraphs: vec![Paragraph::default()],
            ..TextBody::default()
        }));
    }
    s
}

/// One slide with a text box (`spid` 2) and a picture-like shape (`spid` 3).
fn deck() -> SlideParts {
    let mut root = ShapeNode::new(
        ObjectId {
            part: SLIDE,
            node: 1,
        },
        NodeKind::Group,
    );
    root.children.push(shape(2, 2, true));
    root.children.push(shape(3, 3, false));
    let mut part = SlidePart::new(ObjectId::part(SLIDE), SlideKind::Slide);
    part.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
        controls: Vec::new(),
    });
    SlideParts {
        structure: DeckStructure {
            parts: vec![PartEntry {
                id: ObjectId::part(SLIDE),
                kind: SlideKind::Slide,
                part: "ppt/slides/slide1.xml".to_owned(),
                layout: None,
                master: None,
                notes: None,
                theme: None,
            }],
            slides: vec![SLIDE],
            ..DeckStructure::default()
        },
        parts: vec![Some(part)],
    }
}

fn add(index: usize, spid: u32, kind: EffectKind, start: EffectStart) -> Operation {
    Operation::AddEffect {
        ordinal: SLIDE,
        index,
        effect: NewEffect {
            spid,
            kind,
            start,
            duration_ms: kind.default_ms(),
            option: None,
            delay_ms: 0,
        },
    }
}

fn listed(parts: &SlideParts) -> Vec<EffectEntry> {
    effects(SLIDE, parts.get(SLIDE).unwrap().timing.as_ref()).unwrap()
}

/// Three effects: Fade on click, Fly In with it, Fade Out after them.
fn animated() -> SlideParts {
    let mut parts = deck();
    for op in [
        add(0, 2, EffectKind::Fade, EffectStart::OnClick),
        add(1, 3, EffectKind::FlyIn, EffectStart::WithPrevious),
        add(2, 2, EffectKind::FadeOut, EffectStart::AfterPrevious),
    ] {
        apply(&mut parts, op).unwrap();
    }
    parts
}

#[test]
fn each_motion_op_applied_then_its_inverse_gives_the_model_back() {
    let fade = build_transition(TransitionKind::Fade, 700, None);
    for (start, op) in [
        (
            deck(),
            Operation::SetTransition {
                ordinal: SLIDE,
                transition: fade.clone(),
            },
        ),
        (
            deck(),
            Operation::SetTransition {
                ordinal: SLIDE,
                transition: build_transition(TransitionKind::Cut, 0, None),
            },
        ),
        (deck(), add(0, 2, EffectKind::Appear, EffectStart::OnClick)),
        (
            deck(),
            add(0, 3, EffectKind::Pulse, EffectStart::AfterPrevious),
        ),
        (
            animated(),
            add(1, 3, EffectKind::Disappear, EffectStart::OnClick),
        ),
        (
            animated(),
            Operation::RemoveEffect {
                ordinal: SLIDE,
                index: 0,
            },
        ),
        (
            animated(),
            Operation::RemoveEffect {
                ordinal: SLIDE,
                index: 2,
            },
        ),
        (
            animated(),
            Operation::MoveEffect {
                ordinal: SLIDE,
                from: 2,
                to: 0,
            },
        ),
        (
            animated(),
            Operation::SetEffectStart {
                ordinal: SLIDE,
                index: 1,
                start: EffectStart::OnClick,
            },
        ),
        (
            animated(),
            Operation::SetTiming {
                ordinal: SLIDE,
                timing: None,
            },
        ),
    ] {
        let mut parts = start.clone();
        let inverse = apply(&mut parts, op.clone()).unwrap();
        assert_ne!(parts, start, "{op:?} changed nothing");
        let again = apply(&mut parts, inverse).unwrap();
        assert_eq!(parts, start, "{op:?} not undone");
        let mut redone = start.clone();
        apply(&mut redone, op.clone()).unwrap();
        let mut via_inverse = start.clone();
        apply(&mut via_inverse, again).unwrap();
        assert_eq!(redone, via_inverse, "{op:?} redo differs");
    }
}

#[test]
fn the_pane_lists_effects_in_order_with_their_start_and_kind() {
    let parts = animated();
    let got: Vec<_> = listed(&parts)
        .iter()
        .map(|e| (e.spid, e.kind, e.start, e.duration_ms))
        .collect();
    assert_eq!(
        got,
        vec![
            (Some(2), Some(EffectKind::Fade), EffectStart::OnClick, 500),
            (
                Some(3),
                Some(EffectKind::FlyIn),
                EffectStart::WithPrevious,
                500
            ),
            (
                Some(2),
                Some(EffectKind::FadeOut),
                EffectStart::AfterPrevious,
                500
            ),
        ]
    );
    assert_eq!(listed(&parts)[2].class, Some(TimeNodePresetClass::Exit));
}

fn main_clicks(parts: &SlideParts) -> Vec<TimeNode> {
    let Some(Slot::Plain(t)) = &parts.get(SLIDE).unwrap().timing else {
        panic!("plain timing");
    };
    let TimeNode::Par(root) = &t.tn_lst.as_ref().unwrap()[0] else {
        panic!("root par")
    };
    let root = root.common.as_ref().unwrap();
    assert_eq!(root.node_type, Some(TimeNodeType::Root));
    assert_eq!(root.node_id, Some(1));
    let TimeNode::Seq(seq) = &root.child_tn_lst.as_ref().unwrap()[0] else {
        panic!("main sequence")
    };
    let c = seq.common.as_ref().unwrap();
    assert_eq!(c.node_type, Some(TimeNodeType::MainSequence));
    assert_eq!(c.node_id, Some(2));
    c.child_tn_lst.clone().unwrap()
}

fn kids(node: &TimeNode) -> Vec<TimeNode> {
    let TimeNode::Par(p) = node else {
        panic!("par")
    };
    p.common.as_ref().unwrap().child_tn_lst.clone().unwrap()
}

fn start_delay(node: &TimeNode) -> Option<TimeValue> {
    let TimeNode::Par(p) = node else {
        panic!("par")
    };
    p.common.as_ref().unwrap().st_cond_lst.as_ref().unwrap()[0].delay
}

#[test]
fn effects_group_by_click_and_after_steps_wait_for_the_ones_before() {
    let parts = animated();
    let clicks = main_clicks(&parts);
    assert_eq!(clicks.len(), 1, "one click");
    assert_eq!(start_delay(&clicks[0]), Some(TimeValue::Indefinite));
    let steps = kids(&clicks[0]);
    assert_eq!(
        steps.len(),
        2,
        "with joins the click's step, after starts one"
    );
    assert_eq!(kids(&steps[0]).len(), 2);
    assert_eq!(start_delay(&steps[0]), Some(TimeValue::Milliseconds(0)));
    // The first step runs 500 ms: the after step waits that long.
    assert_eq!(start_delay(&steps[1]), Some(TimeValue::Milliseconds(500)));
}

#[test]
fn a_first_effect_not_on_click_starts_with_the_slide() {
    let mut parts = deck();
    apply(
        &mut parts,
        add(0, 3, EffectKind::Appear, EffectStart::AfterPrevious),
    )
    .unwrap();
    let clicks = main_clicks(&parts);
    let TimeNode::Par(p) = &clicks[0] else {
        panic!()
    };
    let conds = p.common.as_ref().unwrap().st_cond_lst.clone().unwrap();
    assert_eq!(conds.len(), 2);
    // `onBegin` of the main sequence, by its id after numbering.
    assert_eq!(conds[1].trigger, Some(ConditionTrigger::Node(Some(2))));
}

#[test]
fn ids_are_numbered_in_order_and_unique() {
    let parts = animated();
    let mut ids = Vec::new();
    struct Ids<'a>(&'a mut Vec<u32>);
    impl casual_slide_model::timing::TargetVisitor for Ids<'_> {
        fn shape(&mut self, _: &'static str, _: &mut u32) {}
        fn common(&mut self, n: &mut casual_slide_model::timing::CommonTimeNode) {
            self.0.push(n.node_id.unwrap());
        }
    }
    let Some(Slot::Plain(mut t)) = parts.get(SLIDE).unwrap().timing.clone() else {
        panic!()
    };
    t.visit(&mut Ids(&mut ids));
    let expected: Vec<u32> = (1..=u32::try_from(ids.len()).unwrap()).collect();
    assert_eq!(ids, expected);
}

#[test]
fn a_shape_with_text_is_listed_for_building_until_its_last_effect_goes() {
    let mut parts = animated();
    let builds = |p: &SlideParts| -> Vec<u32> {
        let Some(Slot::Plain(t)) = &p.get(SLIDE).unwrap().timing else {
            return Vec::new();
        };
        t.bld_lst
            .iter()
            .flatten()
            .filter_map(|b| match b {
                BuildEntry::Paragraph(p) => p.target.spid,
                _ => None,
            })
            .collect()
    };
    // The text box, once, and not the shape without text.
    assert_eq!(builds(&parts), vec![2]);
    apply(
        &mut parts,
        Operation::RemoveEffect {
            ordinal: SLIDE,
            index: 0,
        },
    )
    .unwrap();
    assert_eq!(builds(&parts), vec![2], "the fade out still names it");
    apply(
        &mut parts,
        Operation::RemoveEffect {
            ordinal: SLIDE,
            index: 1,
        },
    )
    .unwrap();
    assert!(builds(&parts).is_empty());
}

#[test]
fn removing_the_last_effect_removes_the_timing() {
    let mut parts = deck();
    apply(
        &mut parts,
        add(0, 2, EffectKind::Fade, EffectStart::OnClick),
    )
    .unwrap();
    apply(
        &mut parts,
        Operation::RemoveEffect {
            ordinal: SLIDE,
            index: 0,
        },
    )
    .unwrap();
    assert_eq!(parts.get(SLIDE).unwrap().timing, None);
}

#[test]
fn what_cannot_be_edited_is_refused_and_changes_nothing() {
    let start = animated();
    for (op, code) in [
        (
            add(0, 99, EffectKind::Fade, EffectStart::OnClick),
            "CS-TXN-0009",
        ),
        (
            add(4, 2, EffectKind::Fade, EffectStart::OnClick),
            "CS-TXN-0004",
        ),
        (
            Operation::RemoveEffect {
                ordinal: SLIDE,
                index: 3,
            },
            "CS-TXN-0004",
        ),
        (
            Operation::MoveEffect {
                ordinal: SLIDE,
                from: 0,
                to: 3,
            },
            "CS-TXN-0004",
        ),
        (
            Operation::SetTransition {
                ordinal: 7,
                transition: None,
            },
            "CS-TXN-0007",
        ),
    ] {
        let mut parts = start.clone();
        assert_eq!(apply(&mut parts, op).unwrap_err().code(), code);
        assert_eq!(parts, start);
    }
    let mut unread = deck();
    unread.get_mut(SLIDE).unwrap().timing = Some(Slot::Unread);
    let before = unread.clone();
    let err = apply(
        &mut unread,
        add(0, 2, EffectKind::Fade, EffectStart::OnClick),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0008");
    assert_eq!(unread, before);
}

#[test]
fn a_transition_with_a_duration_is_written_as_powerpoint_writes_it() {
    let t = build_transition(TransitionKind::Push, 1200, None).unwrap();
    let Slot::Alternate(a) = &t else {
        panic!("p14:dur needs a branch requiring p14")
    };
    let choice = a.branches[0].value.as_ref().unwrap();
    assert_eq!(a.branches[0].requires.as_deref(), Some("p14"));
    assert_eq!(choice.p14_dur.as_deref(), Some("1200"));
    let fallback = a.branches[1].value.as_ref().unwrap();
    assert_eq!(fallback.p14_dur, None);
    assert_eq!(TransitionKind::of(fallback), Some(TransitionKind::Push));
    // None removes it; a cut has no duration.
    assert_eq!(build_transition(TransitionKind::None, 0, Some(&t)), None);
    assert!(matches!(
        build_transition(TransitionKind::Cut, 900, None),
        Some(Slot::Plain(_))
    ));
}

#[test]
fn every_transition_kind_and_effect_option_reads_back_as_written() {
    use super::animate::{TransitionOption, build_transition_with, tab_view};
    for code in 0..10 {
        let kind = TransitionKind::from_code(code).unwrap();
        assert_eq!(kind.code(), code);
        for &option in kind.options() {
            let Some(t) = build_transition_with(kind, Some(option), 800, None) else {
                panic!("{kind:?} wrote nothing")
            };
            let read = tab_view(&t).unwrap();
            assert_eq!(TransitionKind::of(read), Some(kind), "{kind:?}");
            assert_eq!(
                TransitionKind::option_of(read),
                Some(option),
                "{kind:?} {option:?}"
            );
            assert_eq!(TransitionOption::from_code(option.code()), Some(option));
        }
    }
    assert_eq!(TransitionKind::from_code(10), None);
}

#[test]
fn reveal_is_written_in_the_p14_branch_with_a_fade_fallback() {
    use super::animate::{TransitionOption, build_transition_with};
    use casual_slide_model::transition::TransitionEffect;
    let t = build_transition_with(
        TransitionKind::Reveal,
        Some(TransitionOption::ThroughBlackRight),
        1000,
        None,
    )
    .unwrap();
    let Slot::Alternate(a) = &t else {
        panic!("reveal needs p14")
    };
    let choice = a.branches[0].value.as_ref().unwrap();
    assert!(matches!(choice.effect, Some(TransitionEffect::Reveal(_))));
    let fallback = a.branches[1].value.as_ref().unwrap();
    assert!(matches!(fallback.effect, Some(TransitionEffect::Fade(_))));
}

#[test]
fn advance_after_and_on_click_are_set_on_every_branch_and_kept_by_a_new_effect() {
    use super::animate::{build_transition_with, tab_view, with_advance};
    let fade = build_transition(TransitionKind::Fade, 700, None);
    let timed = with_advance(fade.as_ref(), false, Some(3000)).unwrap();
    let Slot::Alternate(a) = &timed else {
        panic!("still the two branches")
    };
    for b in &a.branches {
        let v = b.value.as_ref().unwrap();
        assert_eq!(v.adv_click, Some(false));
        assert_eq!(v.adv_tm, Some(3000));
    }
    // Choosing another effect keeps the timing.
    let pushed = build_transition_with(TransitionKind::Push, None, 1000, Some(&timed)).unwrap();
    assert_eq!(tab_view(&pushed).unwrap().adv_tm, Some(3000));
    // Back to the defaults on a slide with no effect leaves no transition.
    let bare = with_advance(None, false, Some(2000));
    assert!(bare.is_some());
    assert_eq!(with_advance(bare.as_ref(), true, None), None);
    // One undoable op.
    let mut parts = deck();
    let start = parts.clone();
    let inverse = apply(
        &mut parts,
        Operation::SetTransition {
            ordinal: SLIDE,
            transition: Some(timed),
        },
    )
    .unwrap();
    assert_ne!(parts, start);
    apply(&mut parts, inverse).unwrap();
    assert_eq!(parts, start);
}

#[test]
fn every_effect_and_its_options_read_back_as_added() {
    use super::animate::EffectOption;
    for code in 0..12 {
        let kind = EffectKind::from_code(code).unwrap();
        assert_eq!(kind.code(), code);
        let options: Vec<Option<EffectOption>> = if kind.options().is_empty() {
            vec![None]
        } else {
            kind.options().iter().copied().map(Some).collect()
        };
        for option in options {
            let mut parts = deck();
            let mut effect = NewEffect::new(3, kind, EffectStart::OnClick);
            effect.option = option;
            apply(
                &mut parts,
                Operation::AddEffect {
                    ordinal: SLIDE,
                    index: 0,
                    effect,
                },
            )
            .unwrap();
            let e = &listed(&parts)[0];
            assert_eq!(e.kind, Some(kind), "{kind:?}");
            assert_eq!(e.option, option, "{kind:?}");
        }
    }
    assert_eq!(EffectKind::from_code(12), None);
}

#[test]
fn set_effect_changes_option_duration_and_delay_in_one_undo_step() {
    use super::animate::EffectOption;
    let mut parts = animated();
    let start = parts.clone();
    let before = listed(&parts);
    let mut effect = NewEffect::new(3, EffectKind::FlyIn, before[1].start);
    effect.option = Some(EffectOption::FromLeft);
    effect.duration_ms = 1250;
    effect.delay_ms = 400;
    let inverse = apply(
        &mut parts,
        Operation::SetEffect {
            ordinal: SLIDE,
            index: 1,
            effect,
        },
    )
    .unwrap();
    let after = listed(&parts);
    assert_eq!(after.len(), before.len());
    assert_eq!(after[1].kind, Some(EffectKind::FlyIn));
    assert_eq!(after[1].option, Some(EffectOption::FromLeft));
    assert_eq!(after[1].duration_ms, 1250);
    assert_eq!(after[1].delay_ms, 400);
    assert_eq!(after[0], before[0]);
    apply(&mut parts, inverse).unwrap();
    assert_eq!(parts, start);
}
