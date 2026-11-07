//! Tier 1I import: `p:timing` and `p:transition` read whole, in `mc:AlternateContent` with
//! both branches, every limit leaving the deck open, targets by `spid`, nothing read past.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{
    ImportedSlides, SlideLimits, import_slide_part, import_slide_parts,
};
use casual_slide_import::timing::{TimingFinding as F, TimingLimits};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::timing::*;
use casual_slide_model::transition::*;
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

const PART: &str = "ppt/slides/slide1.xml";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(path: &str) -> Vec<u8> {
    std::fs::read(repo().join("fixtures").join(path)).unwrap()
}

fn load_with(path: &str, limits: &SlideLimits) -> ImportedSlides {
    let mut pkg = PresentationPackage::open(fixture(path), OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        limits,
        &ImportLimits::default(),
    )
    .unwrap()
}

fn load(path: &str) -> ImportedSlides {
    load_with(path, &SlideLimits::default())
}

fn slides(deck: &ImportedSlides) -> Vec<&SlidePart> {
    deck.parts
        .structure
        .slides
        .iter()
        .filter_map(|&o| deck.parts.get(o))
        .collect()
}

fn first(deck: &ImportedSlides) -> &SlidePart {
    slides(deck)[0]
}

fn plain_timing(part: &SlidePart) -> &SlideTiming {
    match part.timing.as_ref().unwrap() {
        Slot::Plain(timing) => timing,
        other => panic!("{other:?}"),
    }
}

/// Every node in a list, what they hold included.
fn walk<'a>(list: &'a [TimeNode], out: &mut Vec<&'a TimeNode>) {
    for node in list {
        out.push(node);
        let common = match node {
            TimeNode::Par(n) | TimeNode::Excl(n) => n.common.as_ref(),
            TimeNode::Seq(n) => n.common.as_ref(),
            TimeNode::Alternate(a) => {
                for branch in &a.branches {
                    walk(&branch.value, out);
                }
                None
            }
            _ => None,
        };
        if let Some(common) = common {
            walk(common.child_tn_lst.as_deref().unwrap_or_default(), out);
            walk(common.sub_tn_lst.as_deref().unwrap_or_default(), out);
        }
    }
}

fn kinds_in(list: &[TimeNode]) -> BTreeMap<&'static str, usize> {
    let mut all = Vec::new();
    walk(list, &mut all);
    let mut counts = BTreeMap::new();
    for node in all {
        let kind = match node {
            TimeNode::Par(_) => "par",
            TimeNode::Seq(_) => "seq",
            TimeNode::Excl(_) => "excl",
            TimeNode::Anim(_) => "anim",
            TimeNode::AnimClr(_) => "animClr",
            TimeNode::AnimEffect(_) => "animEffect",
            TimeNode::AnimMotion(_) => "animMotion",
            TimeNode::AnimRot(_) => "animRot",
            TimeNode::AnimScale(_) => "animScale",
            TimeNode::Cmd(_) => "cmd",
            TimeNode::Set(_) => "set",
            TimeNode::Audio(_) => "audio",
            TimeNode::Video(_) => "video",
            TimeNode::Alternate(_) => "alternate",
        };
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts
}

#[test]
fn every_kind_of_node_and_behaviour_is_read_with_what_it_says() {
    let deck = load("generated/timing-every-node.pptx");
    let timing = plain_timing(first(&deck));
    let list = timing.tn_lst.as_deref().unwrap();
    assert_eq!(list.len(), 1);
    let kinds = kinds_in(list);
    for name in [
        "par",
        "seq",
        "excl",
        "anim",
        "animClr",
        "animEffect",
        "animMotion",
        "animRot",
        "animScale",
        "cmd",
        "set",
        "audio",
        "video",
    ] {
        assert!(kinds.contains_key(name), "{name}: {kinds:?}");
    }
    let mut all = Vec::new();
    walk(list, &mut all);

    let TimeNode::Par(root) = &list[0] else {
        panic!()
    };
    let root = root.common.as_ref().unwrap();
    assert_eq!(root.node_type, Some(TimeNodeType::Root));
    assert_eq!(root.dur, Some(TimeValue::Indefinite));
    assert_eq!(root.restart, Some(TimeNodeRestart::Never));

    let full = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Par(p) => p.common.as_ref().filter(|c| c.preset_id == Some(2)),
            _ => None,
        })
        .unwrap();
    assert_eq!(full.preset_class, Some(TimeNodePresetClass::Exit));
    assert_eq!(full.preset_subtype, Some(1));
    assert_eq!(full.dur, Some(TimeValue::Milliseconds(100)));
    assert_eq!(full.repeat_count, Some(TimeValue::Milliseconds(1001)));
    assert_eq!(full.repeat_dur, Some(TimeValue::Milliseconds(2)));
    assert_eq!(full.spd.unwrap().0, 100_000);
    assert_eq!(full.accel.unwrap().0, 10_000);
    assert_eq!(full.decel.unwrap().0, 20_000);
    assert_eq!(full.auto_rev, Some(true));
    assert_eq!(full.restart, Some(TimeNodeRestart::WhenNotActive));
    assert_eq!(full.fill, Some(TimeNodeFill::Freeze));
    assert_eq!(full.sync_behavior, Some(TimeNodeSync::Locked));
    assert_eq!(full.tm_filter.as_deref(), Some("0,0;1,1"));
    assert_eq!(full.evt_filter.as_deref(), Some("cancelBubble"));
    assert_eq!(full.display, Some(false));
    assert_eq!(full.master_rel, Some(TimeNodeMasterRelation::NextClick));
    assert_eq!(full.bld_lvl, Some(1));
    assert_eq!(full.grp_id, Some(0));
    assert_eq!(full.after_effect, Some(true));
    assert_eq!(full.node_type, Some(TimeNodeType::WithEffect));
    assert_eq!(full.node_ph, Some(true));
    assert_eq!(
        full.st_cond_lst.as_ref().unwrap()[0].delay,
        Some(TimeValue::Milliseconds(0))
    );
    assert_eq!(
        full.end_cond_lst.as_ref().unwrap()[0].evt,
        Some(TriggerEvent::End)
    );
    let sync = full.end_sync.as_ref().unwrap();
    assert_eq!(sync.evt, Some(TriggerEvent::OnEnd));
    assert_eq!(
        sync.trigger,
        Some(ConditionTrigger::Runtime(Some(TriggerRuntimeNode::All)))
    );
    let iterate = full.iterate.unwrap();
    assert_eq!(iterate.backwards, Some(true));
    assert_eq!(
        iterate.interval,
        Some(IterateInterval::Time(Some(TimeValue::Milliseconds(100))))
    );
    assert!(full.sub_tn_lst.is_some() && full.child_tn_lst.is_some());

    let motion = all
        .iter()
        .find_map(|n| match n {
            TimeNode::AnimMotion(m) => Some(m),
            _ => None,
        })
        .unwrap();
    assert_eq!(motion.path.as_deref(), Some("M 0 0 L 0.5 0.5 E"));
    assert_eq!(motion.origin, Some(AnimateMotionOrigin::Parent));
    assert_eq!(
        motion.path_edit_mode,
        Some(AnimateMotionPathEditMode::Relative)
    );
    assert_eq!(motion.r_ang.unwrap().0, 60_000);
    assert_eq!(motion.pts_types.as_deref(), Some("AA"));
    assert_eq!(motion.to.unwrap().x.unwrap().0, 50_000);
    assert_eq!(motion.r_ctr.unwrap().y.unwrap().0, -100);

    let color = all
        .iter()
        .find_map(|n| match n {
            TimeNode::AnimClr(c) if c.from.is_some() => Some(c),
            _ => None,
        })
        .unwrap();
    assert_eq!(color.clr_spc, Some(AnimateColorSpace::Rgb));
    assert!(matches!(color.from, Some(Some(_))) && matches!(color.to, Some(Some(_))));
    assert!(all.iter().any(|n| matches!(
        n,
        TimeNode::AnimClr(AnimateColorBehavior {
            by: Some(ByAnimateColor::Hsl(_)),
            ..
        })
    )));

    let effect = all
        .iter()
        .find_map(|n| match n {
            TimeNode::AnimEffect(e) => Some(e),
            _ => None,
        })
        .unwrap();
    assert_eq!(effect.filter.as_deref(), Some("fade"));
    assert_eq!(effect.pr_lst.as_deref(), Some("gradientSize:0.5"));
    assert!(matches!(effect.progress, Some(AnimVariant::Float(Some(_)))));

    let command = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Cmd(c) => Some(c),
            _ => None,
        })
        .unwrap();
    assert_eq!(command.command_type, Some(CommandType::Event));
    assert_eq!(command.cmd.as_deref(), Some("playFrom(0.0)"));

    let anim = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Anim(a) if a.tav_lst.is_some() => Some(a),
            _ => None,
        })
        .unwrap();
    let values = anim.tav_lst.as_ref().unwrap();
    assert_eq!(values.len(), 7);
    assert_eq!(
        values[1].tm,
        Some(AnimateValueTime::At(
            casual_slide_model::presentation::Percentage(10_000)
        ))
    );
    assert_eq!(values[6].tm, Some(AnimateValueTime::Indefinite));
    assert_eq!(values[1].fmla.as_deref(), Some("#ppt_x+1"));
    assert_eq!(values[0].val, Some(AnimVariant::Bool(Some(true))));
    assert_eq!(values[1].val, Some(AnimVariant::Int(Some(-5))));
    assert!(matches!(values[2].val, Some(AnimVariant::Float(Some(_)))));
    assert_eq!(
        values[3].val,
        Some(AnimVariant::Str(Some("visible".to_owned())))
    );
    assert!(matches!(values[4].val, Some(AnimVariant::Color(Some(_)))));

    let audio = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Audio(a) => Some(a),
            _ => None,
        })
        .unwrap();
    assert_eq!(audio.is_narration, Some(true));
    let media = audio.media.as_ref().unwrap();
    assert_eq!(media.vol.unwrap().0, 80_000);
    assert_eq!(media.num_sld, Some(2));
    assert_eq!(media.show_when_stopped, Some(false));

    let builds = timing.bld_lst.as_ref().unwrap();
    assert!(builds.iter().any(|b| matches!(b, BuildEntry::Diagram(_))));
    assert!(builds.iter().any(|b| matches!(b, BuildEntry::OleChart(_))));
    let BuildEntry::Paragraph(first_build) = &builds[0] else {
        panic!()
    };
    assert_eq!(first_build.target.spid, Some(3));
    assert_eq!(first_build.target.ui_expand, Some(true));
    assert_eq!(first_build.adv_auto, Some(TimeValue::Milliseconds(1000)));
    assert!(builds.iter().any(|b| matches!(
        b,
        BuildEntry::Graphic(BuildGraphic {
            choice: Some(GraphicBuildChoice::AsOne),
            ..
        })
    )));
    assert!(builds.iter().any(|b| matches!(
        b,
        BuildEntry::Graphic(BuildGraphic {
            choice: Some(GraphicBuildChoice::Sub(
                GraphicalObjectBuildProperties::Chart(_)
            )),
            ..
        })
    )));
    assert!(builds.iter().any(|b| matches!(
        b,
        BuildEntry::Paragraph(BuildParagraph {
            tmpl_lst: Some(list),
            ..
        }) if list.len() == 2 && list[1].lvl == Some(1)
    )));

    // Valid, so nothing but `extLst` goes unmodelled.
    let unexpected: Vec<_> = deck
        .findings
        .iter()
        .filter(|f| f.part == PART && f.element != "extLst")
        .collect();
    assert!(unexpected.is_empty(), "{unexpected:?}");
    assert!(deck.findings.has(K::RetainedElement, "extLst"));
}

#[test]
fn every_target_kind_is_read_in_a_condition() {
    let deck = load("generated/timing-every-node.pptx");
    let timing = plain_timing(first(&deck));
    let mut all = Vec::new();
    walk(timing.tn_lst.as_deref().unwrap(), &mut all);
    let conditions = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Par(p) => p
                .common
                .as_ref()
                .and_then(|c| c.st_cond_lst.as_ref())
                .filter(|l| l.len() > 5),
            _ => None,
        })
        .unwrap();
    let targets: Vec<&TimeTarget> = conditions
        .iter()
        .filter_map(|c| match &c.trigger {
            Some(ConditionTrigger::Target(t)) => Some(t),
            _ => None,
        })
        .collect();
    assert!(targets.iter().any(|t| matches!(t, TimeTarget::Slide)));
    assert!(targets.iter().any(|t| matches!(
        t,
        TimeTarget::Sound(EmbeddedSound { rel_id: Some(id), name: Some(name), .. }) if id == "rId2" && name == "ding.wav"
    )));
    assert!(
        targets
            .iter()
            .any(|t| matches!(t, TimeTarget::Ink(Some(_))))
    );
    let subs: Vec<&ShapeSubTarget> = targets
        .iter()
        .filter_map(|t| match t {
            TimeTarget::Shape(s) => s.sub.as_ref(),
            _ => None,
        })
        .collect();
    assert!(subs.iter().any(|s| matches!(s, ShapeSubTarget::Background)));
    assert!(
        subs.iter()
            .any(|s| matches!(s, ShapeSubTarget::SubShape(Some(_))))
    );
    assert!(subs.iter().any(|s| matches!(
        s,
        ShapeSubTarget::OleChart(OleChartTarget {
            element_type: Some(_),
            lvl: Some(_)
        })
    )));
    assert!(
        subs.iter()
            .any(|s| matches!(s, ShapeSubTarget::Text(TextTarget::Whole)))
    );
    assert!(subs.iter().any(|s| matches!(
        s,
        ShapeSubTarget::Text(TextTarget::Characters(IndexRange {
            start: Some(0),
            end: Some(1)
        }))
    )));
    assert!(
        subs.iter()
            .any(|s| matches!(s, ShapeSubTarget::Text(TextTarget::Paragraphs(_))))
    );
    assert!(subs.iter().any(|s| matches!(
        s,
        ShapeSubTarget::Graphic(AnimationElementChoice::Diagram(AnimationDiagramElement {
            id: Some(_),
            build_step: Some(_)
        }))
    )));
    assert!(subs.iter().any(|s| matches!(
        s,
        ShapeSubTarget::Graphic(AnimationElementChoice::Chart(AnimationChartElement {
            build_step: Some(_),
            ..
        }))
    )));
    assert!(
        conditions
            .iter()
            .any(|c| matches!(c.trigger, Some(ConditionTrigger::Node(Some(2)))))
    );
}

/// All 42: 21 base, 19 from the 2010 namespace, `prstTrans` and Morph.
#[test]
fn every_transition_is_read_and_both_branches_are_kept() {
    let mut names = HashSet::new();
    let mut note = |effect: &TransitionEffect| {
        let debug = format!("{effect:?}");
        names.insert(debug.split(['(', ' ']).next().unwrap().to_owned());
    };
    let deck = load("generated/transition-every-effect.pptx");
    let mut alternates = 0;
    for part in slides(&deck) {
        match part.transition.as_ref().unwrap() {
            Slot::Alternate(a) => {
                alternates += 1;
                assert_eq!(a.branches.len(), 2);
                assert!(a.branches[0].is_choice && !a.branches[1].is_choice);
                assert!(a.branches[0].requires.is_some());
                assert_eq!(a.branches[1].requires, None);
                // The resolver understands ECMA-376 alone, so the fallback applies.
                assert_eq!(a.selected, Some(1));
                let fallback = part.transition.as_ref().unwrap().resolved().unwrap();
                assert_eq!(
                    fallback.effect,
                    Some(TransitionEffect::Fade(OptionalBlackTransition {
                        thru_blk: Some(true)
                    }))
                );
                let choice = a.branches[0].value.as_ref().unwrap();
                assert!(choice.effect.as_ref().unwrap().is_extension());
                note(choice.effect.as_ref().unwrap());
            }
            Slot::Plain(t) => {
                assert!(t.effect.is_some());
            }
            Slot::Unread => panic!("unread"),
        }
    }
    let area = load("generated/area-trans.pptx");
    for part in slides(&area) {
        let t = part.transition.as_ref().unwrap().resolved().unwrap();
        note(t.effect.as_ref().unwrap());
        assert!(t.spd.is_some() && t.adv_click == Some(true) && t.adv_tm.is_some());
    }
    assert_eq!(alternates, 61, "every newer effect, each in a branch");
    assert_eq!(names.len(), 42, "{names:?}");
}

#[test]
fn a_transitions_attributes_and_sound_are_read() {
    let deck = load("generated/transition-every-effect.pptx");
    let all = slides(&deck);
    let Some(Slot::Alternate(a)) = &all[0].transition else {
        panic!()
    };
    assert_eq!(
        a.branches[0].value.as_ref().unwrap().p14_dur.as_deref(),
        Some("1000")
    );
    let Some(Slot::Plain(start)) = &all[all.len() - 2].transition else {
        panic!()
    };
    assert_eq!(start.spd, Some(TransitionSpeed::Medium));
    assert_eq!(start.adv_click, Some(false));
    assert_eq!(start.adv_tm, Some(1500));
    let Some(TransitionSoundAction::Start(sound)) = &start.snd_ac else {
        panic!("{:?}", start.snd_ac)
    };
    assert_eq!(sound.looping, Some(true));
    assert_eq!(sound.snd.as_ref().unwrap().rel_id.as_deref(), Some("rId2"));
    let Some(Slot::Plain(end)) = &all[all.len() - 1].transition else {
        panic!()
    };
    assert_eq!(end.snd_ac, Some(TransitionSoundAction::End));
    assert!(
        deck.findings.iter().all(|f| f.element == "extLst"),
        "{:?}",
        deck.findings
    );
}

/// `MS-OI29500` nothing: the older namespace for `dur`, from a real deck.
#[test]
fn the_older_namespace_for_dur_is_read_as_the_released_one() {
    let deck = load("real/sdk-animation.pptx");
    let durations: Vec<&str> = slides(&deck)
        .iter()
        .filter_map(|p| match p.transition.as_ref()? {
            Slot::Alternate(a) => a.branches[0].value.as_ref()?.p14_dur.as_deref(),
            _ => None,
        })
        .collect();
    assert!(!durations.is_empty());
}

#[test]
fn alternate_content_inside_a_tree_and_around_one_keeps_every_branch() {
    let deck = load("generated/timing-alternate-content.pptx");
    let all = slides(&deck);
    let timing = plain_timing(all[0]);
    let mut nodes = Vec::new();
    walk(timing.tn_lst.as_deref().unwrap(), &mut nodes);
    let alternates = nodes
        .iter()
        .find_map(|n| match n {
            TimeNode::Alternate(a) => Some(a),
            _ => None,
        })
        .unwrap();
    assert_eq!(alternates.branches.len(), 2);
    assert_eq!(alternates.selected, Some(1));
    assert_eq!(alternates.branches[0].requires.as_deref(), Some("p14"));
    assert_eq!(alternates.branches[0].value.len(), 1);
    assert_eq!(alternates.resolved().map(Vec::len), Some(1));
    // `p14:presetBounceEnd` is reported, and stays.
    assert!(deck.findings.has(K::RetainedAttribute, "cTn"));

    let Some(Slot::Alternate(whole)) = &all[1].timing else {
        panic!("{:?}", all[1].timing)
    };
    assert_eq!(whole.selected, Some(1));
    assert!(whole.branches.iter().all(|b| b.value.is_some()));
    // A target in the fallback names shape 4, and it is the resolved one.
    let resolved = all[1].timing.as_ref().unwrap().resolved().unwrap();
    let mut clone = resolved.clone();
    struct Seen(Vec<u32>);
    impl TargetVisitor for Seen {
        fn shape(&mut self, _: &'static str, spid: &mut u32) {
            self.0.push(*spid);
        }
    }
    let mut seen = Seen(Vec::new());
    clone.visit(&mut seen);
    assert_eq!(seen.0, vec![4]);
    assert_eq!(all[1].animated_spids(), vec![2, 4]);
}

#[test]
fn strict_namespaces_and_percent_forms_read_the_same() {
    let plain = load("generated/timing-every-node.pptx");
    let strict = load("generated/timing-strict.pptx");
    assert_eq!(first(&plain).timing, first(&strict).timing);
    assert!(matches!(first(&strict).timing, Some(Slot::Plain(_))));
}

/// Over any bound: the whole timing is unread, reported and in the tree, and the slide,
/// and the deck, read as they would.
#[test]
fn over_each_limit_timing_is_unread_and_reported_and_the_deck_opens() {
    let over = K::Timing(F::OverLimit);
    for (file, element) in [
        ("timing-limit-depth", "childTnLst"),
        ("timing-limit-nodes", "tnLst"),
        ("timing-limit-conditions", "stCondLst"),
        ("timing-limit-attr-names", "attrNameLst"),
        ("timing-limit-animate-values", "tavLst"),
        ("timing-limit-builds", "bldLst"),
        ("timing-limit-templates", "tmplLst"),
        ("timing-limit-branches", "AlternateContent"),
    ] {
        let deck = load(&format!("hostile/{file}.bin"));
        let part = first(&deck);
        assert_eq!(part.timing, Some(Slot::Unread), "{file}");
        assert!(
            deck.findings.has(over, element),
            "{file}: {:?}",
            deck.findings
        );
        assert!(deck.findings.has(over, "timing"), "{file}");
        assert_eq!(
            part.shape_tree().unwrap().descendants().count(),
            5,
            "{file}"
        );
    }
    assert_eq!(over.code(), "CS-TMG-0001");
}

/// One short of each bound reads, so the limit is what the fixture crossed. Named one by
/// one, as each is its own field.
#[test]
fn raising_each_limit_by_one_reads_what_it_refused() {
    let raised = |file: &str, timing: TimingLimits| {
        let limits = SlideLimits {
            timing,
            ..SlideLimits::default()
        };
        let deck = load_with(&format!("hostile/{file}.bin"), &limits);
        assert!(
            matches!(first(&deck).timing, Some(Slot::Plain(_))),
            "{file}"
        );
        assert!(
            !deck.findings.has(K::Timing(F::OverLimit), "timing"),
            "{file}"
        );
    };
    let d = TimingLimits::default();
    raised(
        "timing-limit-depth",
        TimingLimits {
            max_node_depth: d.max_node_depth + 1,
            ..d
        },
    );
    raised(
        "timing-limit-nodes",
        TimingLimits {
            max_nodes: 100_000 + 2,
            ..d
        },
    );
    raised(
        "timing-limit-conditions",
        TimingLimits {
            max_conditions: d.max_conditions + 1,
            ..d
        },
    );
    raised(
        "timing-limit-attr-names",
        TimingLimits {
            max_attr_names: d.max_attr_names + 1,
            ..d
        },
    );
    raised(
        "timing-limit-animate-values",
        TimingLimits {
            max_animate_values: d.max_animate_values + 1,
            ..d
        },
    );
    raised(
        "timing-limit-builds",
        TimingLimits {
            max_builds: d.max_builds + 1,
            ..d
        },
    );
    raised(
        "timing-limit-templates",
        TimingLimits {
            max_templates: d.max_templates + 1,
            ..d
        },
    );
    raised(
        "timing-limit-branches",
        TimingLimits {
            max_branches: d.max_branches + 1,
            ..d
        },
    );
}

#[test]
fn a_value_too_long_to_read_leaves_timing_and_transition_unread_not_the_deck() {
    let mut pkg = PresentationPackage::open(
        fixture("hostile/timing-long-value.bin"),
        OoxmlLimits::default(),
    )
    .unwrap();
    let presentation = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck
        .presentation;
    let tight = ImportLimits {
        max_value_bytes: 1 << 15,
        ..ImportLimits::default()
    };
    let deck =
        import_slide_parts(&mut pkg, &presentation, &SlideLimits::default(), &tight).unwrap();
    let part = first(&deck);
    assert_eq!(part.timing, Some(Slot::Unread));
    assert_eq!(part.transition, Some(Slot::Unread));
    let bad = K::Timing(F::Unreadable);
    assert_eq!(bad.code(), "CS-TMG-0002");
    assert!(deck.findings.has(bad, "timing") && deck.findings.has(bad, "transition"));
}

/// Deeper than XML itself allows: the strict reader refuses the part, before anything here
/// sees it. That is that slide's fault, as any slide's (`REC-03`, `REC-04` will read more
/// of it). The deck opens without it.
#[test]
fn timing_past_the_xml_depth_loses_that_slide_at_worst_and_not_the_deck() {
    let deck = load("hostile/timing-beyond-xml-depth.bin");
    assert!(deck.parts.get(deck.parts.structure.slides[0]).is_none());
    assert!(deck.findings.iter().any(|f| f.kind == K::UnreadablePart));
}

#[test]
fn bad_values_are_reported_and_left_out_and_shapes_stay() {
    let deck = load("hostile/timing-bad-values.bin");
    let part = first(&deck);
    let timing = plain_timing(part);
    let mut all = Vec::new();
    walk(timing.tn_lst.as_deref().unwrap(), &mut all);
    assert!(
        all.iter()
            .any(|n| matches!(n, TimeNode::Par(ContainerTimeNode { common: None })))
    );
    let bad = all
        .iter()
        .find_map(|n| match n {
            TimeNode::Par(p) => p
                .common
                .as_ref()
                .filter(|c| c.child_tn_lst.is_some() && c.node_id.is_none()),
            _ => None,
        })
        .unwrap();
    assert_eq!(bad.dur, None);
    assert_eq!(bad.accel, None, "200% is out of ST_PositiveFixedPercentage");
    assert_eq!(bad.decel, None);
    assert_eq!(bad.fill, None);
    let has = |kind, element: &str| deck.findings.has(kind, element);
    assert!(has(K::MalformedValue, "cTn"));
    assert!(has(K::OutOfRange, "cTn"));
    assert!(has(K::RetainedAttribute, "cTn"));
    assert!(has(K::RetainedElement, "foreign") || has(K::RetainedElement, "(unknown)"));
    assert!(has(K::MissingRequired, "par"));
    assert!(has(K::DuplicateElement, "stCondLst"));
    assert!(has(K::DuplicateElement, "endSync"));
    assert!(has(K::MalformedValue, "animClr"));
    assert!(has(K::MissingRequired, "rgb"));
    assert!(has(K::MalformedValue, "spTgt"));
    assert!(has(K::MalformedValue, "bldP"));
    // `tn` then `rtn` in a condition: a choice, and the first stands.
    assert!(has(K::DuplicateElement, "rtn"));
    // In `tgtEl`, `sldTgt` then `inkTgt`.
    assert!(has(K::DuplicateElement, "inkTgt"));
    // A shape stays whatever timing says.
    assert_eq!(part.shape_tree().unwrap().descendants().count(), 5);
}

#[test]
fn a_bad_transition_keeps_what_it_can_and_reports_the_rest() {
    let deck = load("hostile/transition-bad-values.bin");
    let Some(Slot::Plain(t)) = &first(&deck).transition else {
        panic!()
    };
    assert_eq!(t.spd, None);
    assert_eq!(t.adv_click, None);
    assert_eq!(t.adv_tm, None);
    assert_eq!(
        t.effect,
        Some(TransitionEffect::Cover(EightDirectionTransition {
            dir: None
        }))
    );
    assert!(matches!(t.snd_ac, Some(TransitionSoundAction::Start(_))));
    let has = |kind, element: &str| deck.findings.has(kind, element);
    assert!(has(K::MalformedValue, "transition"));
    assert!(has(K::MalformedValue, "cover"));
    assert!(has(K::DuplicateElement, "fade"), "the second effect");
    assert!(has(K::DuplicateElement, "endSnd"));
    assert!(has(K::DuplicateElement, "sndAc"));
    assert!(
        has(K::DuplicateElement, "transition"),
        "the second transition"
    );
    assert!(has(K::MissingRequired, "stSnd"));
}

#[test]
fn a_branch_out_of_order_is_kept_and_the_resolver_decides() {
    let deck = load("hostile/transition-alternate-disorder.bin");
    let Some(Slot::Alternate(a)) = &first(&deck).transition else {
        panic!()
    };
    assert_eq!(a.branches.len(), 2);
    assert!(!a.branches[0].is_choice && a.branches[1].is_choice);
    // A Choice after a Fallback is never selected, and a stray child is reported.
    assert_eq!(a.selected, Some(0));
    assert!(
        deck.findings.has(K::RetainedElement, "foreign")
            || deck.findings.has(K::RetainedElement, "(unknown)")
    );
}

#[test]
fn what_timing_names_and_the_slide_does_not_hold_is_reported_and_kept() {
    let deck = load("hostile/timing-dangling.bin");
    let unknown = K::Timing(F::UnknownShape);
    assert_eq!(unknown.code(), "CS-TMG-0003");
    assert!(deck.findings.has(unknown, "spTgt"));
    assert!(deck.findings.has(unknown, "bldP"));
    assert!(deck.findings.has(K::DuplicateId, "cTn"));
    assert!(deck.findings.has(K::UnresolvedReference, "tn"));
    assert!(deck.findings.has(K::DanglingRelationship, "sndTgt"));
    let mismatch = K::Timing(F::BuildMismatch);
    assert_eq!(mismatch.code(), "CS-TMG-0004");
    assert!(deck.findings.has(mismatch, "bldLst"), "a repeated pair");
    assert!(deck.findings.has(mismatch, "cTn"), "a grpId with no build");
    assert_eq!(first(&deck).animated_spids(), vec![99, 98, 98]);
}

#[test]
fn a_tree_that_is_not_one_par_is_reported_and_kept() {
    let deck = load("hostile/timing-not-one-par.bin");
    let root = K::Timing(F::RootNotParallel);
    assert_eq!(root.code(), "CS-TMG-0005");
    assert!(deck.findings.has(root, "tnLst"));
    assert!(deck.findings.has(K::DuplicateElement, "tnLst"));
    assert!(deck.findings.has(K::DuplicateElement, "timing"));
    let timing = plain_timing(first(&deck));
    assert_eq!(timing.tn_lst.as_ref().unwrap().len(), 2);
}

/// Tier 1D identity: a target's `spid` finds the node, whose id is what ops carry.
#[test]
fn a_target_finds_its_node_and_follows_a_change_of_spid() {
    let deck = load("generated/rr-animated.pptx");
    let part = first(&deck);
    let spids = part.animated_spids();
    assert!(!spids.is_empty());
    for spid in &spids {
        let id = part.animated_node(*spid).unwrap();
        let node = part.node(id).unwrap();
        assert_eq!(node.non_visual.as_ref().unwrap().spid, Some(*spid));
    }
    assert_eq!(part.animated_node(99_999), None);

    let mut moved = part.clone();
    let from = spids[0];
    let id = moved.animated_node(from).unwrap();
    let count = spids.iter().filter(|s| **s == from).count();
    assert_eq!(moved.retarget_shape(from, 77_777), count);
    assert_eq!(
        moved.animated_node(77_777),
        Some(id),
        "the same node, as ids do not move"
    );
    assert_eq!(moved.animated_node(from), None);
    assert!(!moved.animated_spids().contains(&from));
    assert_eq!(moved.retarget_shape(from, 1), 0, "nothing holds it now");
}

/// What real decks use. A deck here is one Office wrote; the fixture says so.
#[test]
fn real_decks_with_animation_read_whole_with_nothing_left_out() {
    let mut read_any = 0;
    for deck in [
        "real/sdk-animation.pptx",
        "real/z21464709-souzou-ja.pptx",
        "real/sdk-strict-animclr.pptx",
    ] {
        let imported = load(deck);
        for part in slides(&imported) {
            let Some(slot) = &part.timing else { continue };
            assert!(!matches!(slot, Slot::Unread), "{deck}");
            read_any += 1;
        }
        let unexpected: Vec<_> = imported
            .findings
            .iter()
            .filter(|f| matches!(f.kind, K::Timing(_)))
            .collect();
        eprintln!("{deck}: {unexpected:?}");
        assert!(
            !imported.findings.has(K::Timing(F::OverLimit), "timing"),
            "{deck}"
        );
    }
    assert!(read_any >= 5, "{read_any}");
}

// --- A silent drop fails here (`docs/06`, per-tier gate 6) ---------------------------

type Report = (SlidePart, Vec<String>, Vec<String>);

fn read(xml: &[u8]) -> Option<Report> {
    let tree = XmlTree::parse(xml, PART, &OoxmlLimits::default()).ok()?;
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        PART,
        &[],
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .ok()?;
    let report = findings
        .iter()
        .map(|f| format!("{} {} {:?}", f.kind.code(), f.element, f.attribute))
        .collect();
    let colors = color
        .items
        .iter()
        .map(|f| format!("{} {} {}", f.kind.code(), f.element, f.detail))
        .collect();
    Some((part, report, colors))
}

fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// Walk from each `transition`, `timing` or branch at the root. Taking away any element or
/// attribute must change the model or the report, or the reader saw past it.
fn silent_in(xml: &[u8], tried: &mut HashSet<(String, String, String)>) -> Vec<String> {
    let whole = XmlTree::parse(xml, PART, &OoxmlLimits::default()).unwrap();
    let as_read = read(xml).unwrap();
    let mut silent = Vec::new();
    let mut stack: Vec<(ElementId, Vec<String>)> = whole
        .child_elements(whole.root())
        .into_iter()
        .filter(|id| {
            matches!(
                local(whole.qualified_name(*id)),
                "transition" | "timing" | "AlternateContent"
            )
        })
        .map(|id| (id, vec![local(whole.qualified_name(id)).to_owned()]))
        .collect();
    while let Some((parent, above)) = stack.pop() {
        for kid in whole.child_elements(parent) {
            let name = whole.qualified_name(kid).to_owned();
            let held = whole.qualified_name(parent).to_owned();
            let mut path = above.clone();
            path.push(local(&name).to_owned());
            stack.push((kid, path));
            // A colour's own reader has its tier's test.
            if above.iter().any(|a| a == "extLst") {
                continue;
            }
            let attrs = whole.attributes(kid, PART).unwrap();
            if tried.insert((held.clone(), name.clone(), String::new())) {
                let mut gone = whole.clone();
                gone.remove_child(parent, kid);
                if read(&gone.to_bytes()).as_ref() == Some(&as_read) {
                    silent.push(format!("<{name}> in <{held}>"));
                }
            }
            for (attr, _) in attrs {
                if attr == "xmlns" || attr.starts_with("xmlns:") {
                    continue;
                }
                if !tried.insert((held.clone(), name.clone(), attr.clone())) {
                    continue;
                }
                let mut gone = whole.clone();
                gone.remove_attribute(kid, &attr);
                if read(&gone.to_bytes()).as_ref() == Some(&as_read) {
                    silent.push(format!("<{name}>/@{attr} in <{held}>"));
                }
            }
        }
    }
    silent
}

fn part_of(deck: &str, part: &str) -> Vec<u8> {
    let mut pkg = PresentationPackage::open(fixture(deck), OoxmlLimits::default()).unwrap();
    pkg.read_part(part).unwrap()
}

#[test]
fn nothing_in_timing_or_a_transition_can_go_without_the_model_or_the_report_changing() {
    let mut silent = Vec::new();
    let mut tried = HashSet::new();
    let mut parts = vec![
        ("generated/timing-every-node.pptx", PART.to_owned()),
        ("generated/timing-alternate-content.pptx", PART.to_owned()),
        (
            "generated/timing-alternate-content.pptx",
            "ppt/slides/slide2.xml".to_owned(),
        ),
        ("real/sdk-animation.pptx", PART.to_owned()),
        (
            "real/z21464709-souzou-ja.pptx",
            "ppt/slides/slide2.xml".to_owned(),
        ),
    ];
    for i in 1..=62 {
        parts.push((
            "generated/transition-every-effect.pptx",
            format!("ppt/slides/slide{i}.xml"),
        ));
    }
    for (deck, part) in &parts {
        let xml = part_of(deck, part);
        silent.extend(
            silent_in(&xml, &mut tried)
                .into_iter()
                .map(|w| format!("{deck} {part}: {w}")),
        );
    }
    assert!(tried.len() > 300, "{}", tried.len());
    assert!(
        silent.is_empty(),
        "{} read past:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// The check itself: a timing element nothing knows is noticed, and so is its going.
#[test]
fn the_silent_drop_check_sees_a_real_drop() {
    let slide = |extra: &str| {
        format!(
            r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld><p:timing{extra}><p:tnLst/></p:timing></p:sld>"#
        )
    };
    let plain = read(slide("").as_bytes()).unwrap();
    let extra = read(slide(r#" madeUp="1""#).as_bytes()).unwrap();
    assert_ne!(plain, extra);
    let mut tried = HashSet::new();
    assert_eq!(
        silent_in(slide(r#" madeUp="1""#).as_bytes(), &mut tried),
        Vec::<String>::new()
    );
}

/// Commands, formulas, paths and filters are file content. None reaches a report.
#[test]
fn no_finding_or_message_carries_text_from_timing() {
    let marker = "SECRETMARKER";
    let xml = format!(
        r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld><p:transition spd="{marker}"><p:{marker}/></p:transition><p:timing {marker}="1"><p:tnLst><p:par><p:cTn id="{marker}" tmFilter="{marker}" presetClass="{marker}"><p:childTnLst><p:cmd type="{marker}" cmd="{marker}"><p:{marker}/></p:cmd><p:anim by="{marker}"><p:cBhvr rctx="{marker}"><p:attrNameLst><p:attrName>{marker}</p:attrName></p:attrNameLst><p:tgtEl><p:sndTgt r:embed="{marker}" name="{marker}"/></p:tgtEl></p:cBhvr></p:anim><p:animMotion path="{marker}" ptsTypes="{marker}"/></p:childTnLst></p:cTn></p:par></p:tnLst></p:timing></p:sld>"#
    );
    let (_, report, _) = read(xml.as_bytes()).unwrap();
    assert!(!report.is_empty());
    for line in &report {
        assert!(!line.contains(marker), "{line}");
    }
}
