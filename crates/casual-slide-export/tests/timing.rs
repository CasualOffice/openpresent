//! Tier 1I export: every row of TIMING and TRANS is in a fixture and comes back from the
//! model; real decks are a fixed point; an edit changes only what it names; a forced rewrite
//! of timing and transitions reads back the same and validates; both branches of
//! `mc:AlternateContent` stay; what would leave a deck wrong is refused with nothing saved.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::slide::{patch_slide_part, save_slide_parts};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::presentation::Percentage;
use casual_slide_model::slide::{SlideKind, SlidePart, SlideParts};
use casual_slide_model::timing::*;
use casual_slide_model::transition::*;
use casual_slide_ooxml::{
    Conformance, OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree,
};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(repo().join("fixtures").join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn model(bytes: &[u8]) -> SlideParts {
    let mut pkg = open(bytes.to_vec());
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let data = pkg.read_part(&n).unwrap();
            (n, data)
        })
        .collect()
}

fn all_decks() -> Vec<String> {
    let mut out = Vec::new();
    for set in ["generated", "producers", "real"] {
        let mut names: Vec<String> = std::fs::read_dir(repo().join("fixtures").join(set))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| !p.extension().is_some_and(|e| e == "tsv" || e == "md"))
            .map(|p| format!("{set}/{}", p.file_name().unwrap().to_string_lossy()))
            .collect();
        names.sort();
        out.extend(names);
    }
    out
}

/// Save one slide with a change, or the code it was refused with.
fn edit_part(
    source: &[u8],
    ordinal: u32,
    change: impl FnOnce(&mut SlidePart),
) -> Result<(Vec<u8>, SlideParts), String> {
    let mut wanted = model(source);
    change(wanted.get_mut(ordinal).unwrap());
    let mut pkg = open(source.to_vec());
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    Ok((outcome.bytes, wanted))
}

fn first_slide(wanted: &SlideParts) -> u32 {
    wanted.structure.slides[0]
}

fn part_name(wanted: &SlideParts, ordinal: u32) -> String {
    wanted.structure.entry(ordinal).unwrap().part.clone()
}

/// Exactly these substitutions between two parts' bytes, and the part is otherwise whole.
fn only_changed(before: &[u8], after: &[u8], changes: &[(&str, &str)]) {
    let mut expected = String::from_utf8(before.to_vec()).unwrap();
    for (from, to) in changes {
        assert!(expected.contains(from), "no {from}");
        expected = expected.replacen(from, to, 1);
    }
    assert_eq!(String::from_utf8(after.to_vec()).unwrap(), expected);
}

/// Which rows have a reason of their own not to be marked. Everything else in the two areas
/// has to be in a fixture and read and written.
fn reason(row: &[&str]) -> Option<&'static str> {
    let (id, kind, name) = (row[0], row[3], row[4]);
    if id.contains("drawing/2018/animation") {
        return Some("animPr is a 3D model's playback, Tier 1L's, and not in p:timing");
    }
    if name == "extLst" {
        return Some("retained, and reported: nothing reads it");
    }
    match kind {
        "simple" | "type" | "group" | "attrgroup" | "base" | "any" => {
            Some("a structural row, held to what it declares by the audit")
        }
        _ => None,
    }
}

/// `docs/spec/checklist.tsv`: a row of TIMING or TRANS with no reason of its own is an
/// element or attribute or value in a fixture this crate rewrites whole, and a row added
/// there with no line here fails rather than passing unseen.
#[test]
fn every_row_of_timing_and_trans_is_in_a_fixture_or_has_a_named_reason() {
    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    let decks = [
        "generated/timing-every-node.pptx",
        "generated/timing-ecma-only.pptx",
        "generated/timing-alternate-content.pptx",
        "generated/transition-every-effect.pptx",
        "generated/area-trans.pptx",
    ];
    let mut xml = String::new();
    for deck in decks {
        for (name, bytes) in parts(&read_fixture(deck)) {
            if name.starts_with("ppt/slides/slide") {
                xml.push_str(&String::from_utf8(bytes).unwrap());
            }
        }
    }
    let (mut seen, mut reasons, mut missing) = (0, BTreeMap::new(), Vec::new());
    for line in text.lines().skip(1) {
        let row: Vec<&str> = line.split('\t').collect();
        if !matches!(row[1], "TIMING" | "TRANS") {
            continue;
        }
        if let Some(why) = reason(&row) {
            *reasons.entry(why).or_insert(0) += 1;
            continue;
        }
        seen += 1;
        let name = row[4];
        let found = match row[3] {
            "attribute" => {
                xml.contains(&format!(" {}=", name.trim_start_matches('@')))
                    || xml.contains(&format!(":{}=", name.trim_start_matches('@')))
            }
            "element" => {
                xml.contains(&format!(":{name}>"))
                    || xml.contains(&format!(":{name} "))
                    || xml.contains(&format!(":{name}/"))
            }
            "root" => xml.contains(&format!(":{}", name.trim_matches(['<', '>']))),
            _ => xml.contains(&format!("=\"{}\"", name.trim_start_matches('='))),
        };
        if !found {
            missing.push(row[0].to_owned());
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
    assert!(seen > 320, "{seen}");
    assert_eq!(reasons.values().sum::<usize>() + seen, 522);
}

/// The forced rewrite of this tier: take each slide's timing and transition away, then ask
/// for them back, so the writer makes every element from the model. It has to read as it
/// did, on every deck, real ones included. Then F4 on what was written.
#[test]
fn every_timing_and_transition_can_be_written_from_the_model() {
    let xml = OoxmlLimits::default();
    let limits = SlideLimits::default();
    let import_limits = ImportLimits::default();
    let mut written = 0;
    for path in all_decks() {
        if path.contains("five-hundred") {
            continue;
        }
        let source = read_fixture(&path);
        let original = model(&source);
        let mut rewritten = 0;
        let mut edited = open(source.clone());
        let strict = edited.conformance() == Conformance::Strict;
        let mut replaced = Vec::new();
        for &ordinal in &original.structure.slides {
            let Some(part) = original.get(ordinal) else {
                continue;
            };
            if part.timing.is_none() && part.transition.is_none() {
                continue;
            }
            let name = original.structure.entry(ordinal).unwrap().part.clone();
            let bytes = edited.read_part(&name).unwrap();
            let rels = edited.relationships_of(&name).to_vec();
            let mut tree = XmlTree::parse(&bytes, &name, &xml).unwrap();
            let mut gone = part.clone();
            gone.timing = None;
            gone.transition = None;
            let patched = |tree: &mut XmlTree, to: &SlidePart| {
                patch_slide_part(
                    tree,
                    &name,
                    strict,
                    &rels,
                    to,
                    &limits,
                    &import_limits,
                    &xml,
                )
            };
            patched(&mut tree, &gone).unwrap_or_else(|e| panic!("{path} {name}: {e}"));
            let stripped = String::from_utf8(tree.to_bytes()).unwrap();
            assert!(!stripped.contains("p:timing>"), "{path} {name}");
            assert!(!stripped.contains("<p:transition"), "{path} {name}");
            patched(&mut tree, part).unwrap_or_else(|e| panic!("{path} {name}: {e}"));
            replaced.push((name, tree.to_bytes()));
            rewritten += 1;
        }
        written += rewritten;
        if rewritten == 0 {
            continue;
        }
        let out = rewritten_deck(&source, &replaced);
        assert_eq!(model(&out), original, "{path}: read back different");
        if !path.starts_with("real/sdk-strict") {
            let (before, after) = (f4_failures(&source), f4_failures(&out));
            assert_eq!(after, before, "{path}: F4");
        }
    }
    assert!(written >= 80, "{written}");
}

/// A deck with these parts replaced, every other entry as it was.
fn rewritten_deck(source: &[u8], replaced: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut pkg = open(source.to_vec());
    let mut edits = casual_slide_package::PartEdits::default();
    for (name, bytes) in replaced {
        edits.replace(name.clone(), bytes.clone());
    }
    pkg.save_with(&edits).unwrap()
}

#[test]
fn every_deck_is_a_fixed_point_and_unedited_bytes_come_back() {
    let mut count = 0;
    for path in all_decks() {
        let source = read_fixture(&path);
        let parts_model = model(&source);
        let mut pkg = open(source.clone());
        let all: Vec<&SlidePart> = parts_model.parts.iter().flatten().collect();
        let outcome = save_slide_parts(
            &mut pkg,
            &parts_model.structure,
            &all,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(outcome.bytes, source, "{path}");
        assert!(outcome.changed_parts.is_empty(), "{path}");
        assert_eq!(model(&outcome.bytes), parts_model, "{path}");
        count += 1;
    }
    assert!(count >= 35, "{count}");
}

/// Hostile decks too: whatever could be read and whatever could not, saving unedited gives
/// the bytes back, unread timing and all, and a second read is the same.
#[test]
fn hostile_timing_stays_as_it_was_through_an_unedited_save() {
    for name in std::fs::read_dir(repo().join("fixtures/hostile")).unwrap() {
        let path = name.unwrap().path();
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        if !(file.starts_with("timing-") || file.starts_with("transition-")) {
            continue;
        }
        let source = std::fs::read(&path).unwrap();
        let parts_model = model(&source);
        let mut pkg = open(source.clone());
        let all: Vec<&SlidePart> = parts_model.parts.iter().flatten().collect();
        let outcome = save_slide_parts(
            &mut pkg,
            &parts_model.structure,
            &all,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(outcome.bytes, source, "{file}");
    }
}

/// Every node a slide's plain timing holds, to change.
fn timing_of(part: &mut SlidePart) -> &mut SlideTiming {
    match part.timing.as_mut().unwrap() {
        Slot::Plain(timing) => timing,
        other => panic!("{other:?}"),
    }
}

/// Into `tmRoot`, `mainSeq`, the click's `par`, and its first effect.
fn deepest_set(list: &mut [TimeNode]) -> Option<&mut SetBehavior> {
    for node in list {
        let common = match node {
            TimeNode::Par(n) | TimeNode::Excl(n) => n.common.as_mut(),
            TimeNode::Seq(n) => n.common.as_mut(),
            TimeNode::Set(n) => return Some(n),
            _ => None,
        };
        if let Some(found) = common
            .and_then(|c| c.child_tn_lst.as_mut())
            .and_then(|l| deepest_set(l))
        {
            return Some(found);
        }
    }
    None
}

#[test]
fn an_edit_to_one_node_changes_only_that_attribute_and_every_other_part_stays() {
    let source = read_fixture("generated/area-timing.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let list = timing_of(part).tn_lst.as_mut().unwrap();
        let set = deepest_set(list).unwrap();
        let common = set.behavior.as_mut().unwrap().common.as_mut().unwrap();
        common.dur = Some(TimeValue::Milliseconds(750));
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
    let (old, new) = (parts(&source), parts(&bytes));
    assert_eq!(old.len(), new.len());
    for (part, data) in &old {
        if *part != name {
            assert_eq!(new.get(part), Some(data), "{part} changed");
        }
    }
    only_changed(&old[&name], &new[&name], &[(r#"dur="1""#, r#"dur="750""#)]);
}

#[test]
fn an_edit_in_a_tree_leaves_what_the_model_does_not_hold_alone() {
    // `p14:presetBounceEnd` and `extLst` are not in the model. They stay.
    let source = read_fixture("generated/timing-alternate-content.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let list = timing_of(part).tn_lst.as_mut().unwrap();
        let TimeNode::Par(root) = &mut list[0] else {
            panic!()
        };
        let root = root.common.as_mut().unwrap();
        root.dur = Some(TimeValue::Indefinite);
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
    let after = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    assert!(after.contains("p14:presetBounceEnd=\"50000\""));
    assert!(after.contains("<mc:Fallback>"));
    only_changed(
        &parts(&source)[&name],
        after.as_bytes(),
        &[(
            r#"<p:cTn id="1" nodeType="tmRoot">"#,
            r#"<p:cTn id="1" nodeType="tmRoot" dur="indefinite">"#,
        )],
    );
}

#[test]
fn an_edit_in_one_branch_keeps_the_other_as_it_was() {
    let source = read_fixture("generated/transition-every-effect.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let Some(Slot::Alternate(a)) = &mut part.transition else {
            panic!()
        };
        a.branches[1].value.as_mut().unwrap().spd = Some(TransitionSpeed::Fast);
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
    only_changed(
        &parts(&source)[&name],
        &parts(&bytes)[&name],
        &[(
            r#"<mc:Fallback><p:transition spd="slow">"#,
            r#"<mc:Fallback><p:transition spd="fast">"#,
        )],
    );
}

#[test]
fn a_transition_changes_in_place_and_goes_back_to_what_it_was() {
    let source = read_fixture("generated/area-trans.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let original = model(&source).get(ordinal).unwrap().transition.clone();
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let Some(Slot::Plain(t)) = &mut part.transition else {
            panic!()
        };
        t.effect = Some(TransitionEffect::Push(SideDirectionTransition {
            dir: Some(SideDirection::Down),
        }));
        t.snd_ac = Some(TransitionSoundAction::End);
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
    let after = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    assert!(
        after.contains(r#"<p:push dir="d"/><p:sndAc><p:endSnd/></p:sndAc></p:transition>"#),
        "{after}"
    );
    let (back, _) = edit_part(&bytes, ordinal, |part| part.transition = original).unwrap();
    assert_eq!(parts(&back)[&name], parts(&source)[&name], "the same bytes");
    f4_validates(&bytes);
}

#[test]
fn a_new_morph_is_written_in_a_branch_with_a_fallback_at_its_place_and_validates() {
    let source = read_fixture("generated/rr-animated.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        part.transition = Some(Slot::Alternate(Alternates {
            branches: vec![
                AlternateBranch {
                    requires: Some("p159".to_owned()),
                    is_choice: true,
                    value: Some(Box::new(SlideTransition {
                        spd: Some(TransitionSpeed::Slow),
                        p14_dur: None,
                        effect: Some(TransitionEffect::Morph(MorphTransition {
                            option: Some(MorphOption::ByWord),
                        })),
                        ..SlideTransition::default()
                    })),
                },
                AlternateBranch {
                    requires: None,
                    is_choice: false,
                    value: Some(Box::new(SlideTransition {
                        effect: Some(TransitionEffect::Fade(OptionalBlackTransition::default())),
                        ..SlideTransition::default()
                    })),
                },
            ],
            selected: Some(1),
        }));
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(model(&bytes), wanted);
    let after = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    let (morph, timing) = (
        after.find("morph").unwrap(),
        after.find("<p:timing").unwrap(),
    );
    assert!(morph < timing, "transition goes before timing");
    assert!(after.contains(r#"Requires="p159""#), "{after}");
    f4_validates(&bytes);
    sdk_passes("new-morph", &bytes);
}

#[test]
fn a_whole_tree_can_be_made_for_a_slide_with_none() {
    let tree = model(&read_fixture("generated/area-timing.pptx"))
        .parts
        .iter()
        .flatten()
        .find_map(|p| p.timing.clone())
        .unwrap();
    let source = read_fixture("generated/three-slides.pptx");
    let wanted = model(&source);
    let ordinal = first_slide(&wanted);
    let spid = wanted.get(ordinal).unwrap().shape_tree().unwrap().children[0]
        .non_visual
        .as_ref()
        .unwrap()
        .spid
        .unwrap();
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let mut slot = tree;
        for value in slot.values_mut() {
            struct To(u32);
            impl TargetVisitor for To {
                fn shape(&mut self, _: &'static str, spid: &mut u32) {
                    *spid = self.0;
                }
            }
            value.visit(&mut To(spid));
        }
        part.timing = Some(slot);
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(model(&bytes), wanted);
    f4_validates(&bytes);
    sdk_passes("new-tree", &bytes);
}

#[test]
fn a_percentage_edit_follows_the_form_it_replaces_and_in_strict_the_schema() {
    // Transitional writes the number; `N%` is kept where the file has it and required in Strict.
    let edit = |list: &mut Vec<TimeNode>| {
        let TimeNode::Par(root) = &mut list[0] else {
            panic!()
        };
        let root = root.common.as_mut().unwrap();
        let TimeNode::Par(first) = &mut root.child_tn_lst.as_mut().unwrap()[0] else {
            panic!()
        };
        first.common.as_mut().unwrap().accel = Some(Percentage(25_000));
    };
    let source = read_fixture("generated/timing-every-node.pptx");
    let ordinal = first_slide(&model(&source));
    let (bytes, _) = edit_part(&source, ordinal, |part| {
        edit(timing_of(part).tn_lst.as_mut().unwrap())
    })
    .unwrap();
    let name = part_name(&model(&source), ordinal);
    only_changed(
        &parts(&source)[&name],
        &parts(&bytes)[&name],
        &[(r#"accel="10000""#, r#"accel="25000""#)],
    );
    let strict = read_fixture("generated/timing-strict.pptx");
    let (bytes, _) = edit_part(&strict, ordinal, |part| {
        edit(timing_of(part).tn_lst.as_mut().unwrap())
    })
    .unwrap();
    only_changed(
        &parts(&strict)[&name],
        &parts(&bytes)[&name],
        &[(r#"accel="10%""#, r#"accel="25%""#)],
    );
    f4_validates(&bytes);
}

fn dangling_edit() -> impl FnOnce(&mut SlidePart) {
    |part| {
        let list = timing_of(part).tn_lst.as_mut().unwrap();
        let set = deepest_set(list).unwrap();
        set.behavior.as_mut().unwrap().target = Some(TimeTarget::Shape(ShapeTarget {
            spid: Some(4242),
            sub: None,
        }));
    }
}

#[test]
fn a_target_naming_no_shape_is_refused_and_a_change_of_spid_takes_its_targets_with_it() {
    let source = read_fixture("generated/area-timing.pptx");
    let wanted = model(&source);
    let ordinal = first_slide(&wanted);
    assert_eq!(
        edit_part(&source, ordinal, dangling_edit()).unwrap_err(),
        "CS-SLE-0005"
    );

    let spid = wanted.get(ordinal).unwrap().animated_spids()[0];
    // The shape alone: its animation would be left naming nothing.
    let id = wanted.get(ordinal).unwrap().animated_node(spid).unwrap();
    let rename_alone = edit_part(&source, ordinal, |part| {
        part.node_mut(id).unwrap().non_visual.as_mut().unwrap().spid = Some(777);
    });
    assert_eq!(rename_alone.unwrap_err(), "CS-SLE-0005");
    // Both, through the model: targets and builds move with the shape, nothing else does.
    let name = part_name(&wanted, ordinal);
    let (bytes, after) = edit_part(&source, ordinal, |part| {
        assert_eq!(part.retarget_shape(spid, 777), 2);
    })
    .unwrap();
    assert_eq!(model(&bytes), after);
    let spid_text = format!("id=\"{spid}\" name");
    only_changed(
        &parts(&source)[&name],
        &parts(&bytes)[&name],
        &[
            (&spid_text, "id=\"777\" name"),
            (
                &format!("<p:spTgt spid=\"{spid}\""),
                "<p:spTgt spid=\"777\"",
            ),
            (&format!("<p:bldP spid=\"{spid}\""), "<p:bldP spid=\"777\""),
        ],
    );
    f4_validates(&bytes);
}

#[test]
fn a_target_the_file_already_held_dangling_does_not_stop_another_edit() {
    let source = read_fixture("hostile/timing-dangling.bin");
    let ordinal = first_slide(&model(&source));
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        let list = timing_of(part).tn_lst.as_mut().unwrap();
        let TimeNode::Par(root) = &mut list[0] else {
            panic!()
        };
        root.common.as_mut().unwrap().dur = Some(TimeValue::Milliseconds(5));
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
}

/// A shape only in a branch the resolver did not take is still a shape. `hostile` is not it:
/// a real shape in `mc:Choice`, targeted, and the edit elsewhere in the tree.
#[test]
fn a_target_on_a_shape_in_the_branch_not_taken_is_not_dangling() {
    const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main""#;
    let xml = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><mc:AlternateContent><mc:Choice Requires="p14"><p:sp><p:nvSpPr><p:cNvPr id="7" name="c"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp></mc:Choice><mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="8" name="f"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp></mc:Fallback></mc:AlternateContent></p:spTree></p:cSld><p:timing><p:bldLst><p:bldP spid="7" grpId="0"/></p:bldLst></p:timing></p:sld>"#
    );
    let part = read_part(xml.as_bytes());
    let mut wanted = part.clone();
    timing_of(&mut wanted)
        .bld_lst
        .as_mut()
        .unwrap()
        .push(BuildEntry::Paragraph(BuildParagraph {
            target: BuildTarget {
                spid: Some(7),
                grp_id: Some(1),
                ui_expand: None,
            },
            ..BuildParagraph::default()
        }));
    let bytes = patch_part(&xml, &wanted).unwrap();
    assert_eq!(read_part(&bytes), wanted);
    let mut bad = part.clone();
    timing_of(&mut bad).bld_lst.as_mut().unwrap()[0] = BuildEntry::Paragraph(BuildParagraph {
        target: BuildTarget {
            spid: Some(404),
            grp_id: Some(0),
            ui_expand: None,
        },
        ..BuildParagraph::default()
    });
    assert_eq!(patch_part(&xml, &bad).unwrap_err(), "CS-SLE-0005");
}

fn read_part(xml: &[u8]) -> SlidePart {
    let tree = XmlTree::parse(xml, "s.xml", &OoxmlLimits::default()).unwrap();
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = casual_slide_import::color::Findings::default();
    import_slide_part(
        &tree,
        "s.xml",
        &sound_rels(),
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .unwrap()
}

fn sound_rels() -> Vec<Relationship> {
    vec![Relationship {
        id: "rId1".to_owned(),
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/audio"
            .to_owned(),
        target: "../media/s.wav".to_owned(),
        mode: TargetMode::Internal,
    }]
}

fn patch_part(xml: &str, to: &SlidePart) -> Result<Vec<u8>, String> {
    let mut tree = XmlTree::parse(xml.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
    patch_slide_part(
        &mut tree,
        "s.xml",
        false,
        &sound_rels(),
        to,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    Ok(tree.to_bytes())
}

const BARE: &str = r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld></p:sld>"#;

#[test]
fn what_would_leave_a_deck_wrong_is_refused_and_nothing_is_saved() {
    let base = read_part(BARE.as_bytes());
    let with = |change: fn(&mut SlidePart)| {
        let mut part = base.clone();
        change(&mut part);
        patch_part(BARE, &part)
    };
    let vortex = || {
        Box::new(SlideTransition {
            effect: Some(TransitionEffect::Vortex(SideDirectionTransition::default())),
            ..SlideTransition::default()
        })
    };

    // A newer effect outside a branch requiring it.
    let mut part = base.clone();
    part.transition = Some(Slot::Plain(vortex()));
    assert_eq!(patch_part(BARE, &part).unwrap_err(), "CS-EXP-0009");
    // In a branch requiring something else.
    let mut part = base.clone();
    part.transition = Some(Slot::Alternate(Alternates {
        branches: vec![AlternateBranch {
            requires: Some("p15".to_owned()),
            is_choice: true,
            value: Some(vortex()),
        }],
        selected: None,
    }));
    assert_eq!(patch_part(BARE, &part).unwrap_err(), "CS-EXP-0009");
    // `p14:dur` outside a branch.
    assert_eq!(
        with(|p| {
            p.transition = Some(Slot::Plain(Box::new(SlideTransition {
                p14_dur: Some("100".to_owned()),
                ..SlideTransition::default()
            })))
        })
        .unwrap_err(),
        "CS-EXP-0009"
    );
    // A branch requiring a prefix nothing declares.
    assert_eq!(
        with(|p| {
            p.timing = Some(Slot::Alternate(Alternates {
                branches: vec![AlternateBranch {
                    requires: Some("nope".to_owned()),
                    is_choice: true,
                    value: Some(Box::default()),
                }],
                selected: None,
            }))
        })
        .unwrap_err(),
        "CS-EXP-0009"
    );
    // Asking for what the reader could not read.
    assert_eq!(
        with(|p| p.timing = Some(Slot::Unread)).unwrap_err(),
        "CS-EXP-0009"
    );
    // The selected branch has to be what the resolver says: the read-back holds it.
    assert_eq!(
        with(|p| {
            p.timing = Some(Slot::Alternate(Alternates {
                branches: vec![
                    AlternateBranch {
                        requires: Some("p14".to_owned()),
                        is_choice: true,
                        value: Some(Box::default()),
                    },
                    AlternateBranch {
                        requires: None,
                        is_choice: false,
                        value: Some(Box::default()),
                    },
                ],
                selected: Some(0),
            }))
        })
        .unwrap_err(),
        "CS-EXP-0007"
    );
    // A tree built by hand deeper than the reader would believe.
    let mut node = TimeNode::Par(ContainerTimeNode::default());
    for _ in 0..TimingBound::DEPTH {
        node = TimeNode::Par(ContainerTimeNode {
            common: Some(CommonTimeNode {
                child_tn_lst: Some(vec![node]),
                ..CommonTimeNode::default()
            }),
        });
    }
    let mut part = base.clone();
    part.timing = Some(Slot::Plain(Box::new(SlideTiming {
        tn_lst: Some(vec![node]),
        bld_lst: None,
    })));
    assert_eq!(patch_part(BARE, &part).unwrap_err(), "CS-EXP-0008");
    // A value the reader would not believe: the read-back refuses, `accel` here.
    let mut part = base.clone();
    part.timing = Some(Slot::Plain(Box::new(SlideTiming {
        tn_lst: Some(vec![TimeNode::Par(ContainerTimeNode {
            common: Some(CommonTimeNode {
                accel: Some(Percentage(500_000)),
                ..CommonTimeNode::default()
            }),
        })]),
        bld_lst: None,
    })));
    assert_eq!(patch_part(BARE, &part).unwrap_err(), "CS-EXP-0007");
    // A sound relationship the part does not have.
    let mut part = base.clone();
    part.transition = Some(Slot::Plain(Box::new(SlideTransition {
        snd_ac: Some(TransitionSoundAction::Start(TransitionStartSound {
            looping: None,
            snd: Some(EmbeddedSound {
                rel_id: Some("rId99".to_owned()),
                ..EmbeddedSound::default()
            }),
        })),
        ..SlideTransition::default()
    })));
    assert_eq!(patch_part(BARE, &part).unwrap_err(), "CS-EXP-0003");
    // And one it does have goes through.
    let mut part = base.clone();
    part.transition = Some(Slot::Plain(Box::new(SlideTransition {
        snd_ac: Some(TransitionSoundAction::Start(TransitionStartSound {
            looping: Some(true),
            snd: Some(EmbeddedSound {
                rel_id: Some("rId1".to_owned()),
                name: Some("s".to_owned()),
                built_in: None,
            }),
        })),
        ..SlideTransition::default()
    })));
    let bytes = patch_part(BARE, &part).unwrap();
    assert_eq!(read_part(&bytes), part);
}

struct TimingBound;
impl TimingBound {
    const DEPTH: usize = 40;
}

/// Unreadable timing can be replaced, and only that: the rest of the part stays.
#[test]
fn unread_timing_can_be_replaced_or_removed_and_nothing_else_moves() {
    let source = read_fixture("hostile/timing-limit-depth.bin");
    let wanted = model(&source);
    let ordinal = first_slide(&wanted);
    assert_eq!(wanted.get(ordinal).unwrap().timing, Some(Slot::Unread));
    let name = part_name(&wanted, ordinal);
    let held = String::from_utf8(parts(&source)[&name].clone()).unwrap();
    let start = held.find("<p:timing>").unwrap();
    let end = held.find("</p:timing>").unwrap() + "</p:timing>".len();

    let (bytes, after) = edit_part(&source, ordinal, |part| {
        part.timing = Some(Slot::Plain(Box::default()));
    })
    .unwrap();
    assert_eq!(model(&bytes), after);
    let text = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    assert_eq!(
        text,
        format!("{}<p:timing/>{}", &held[..start], &held[end..])
    );

    let (bytes, after) = edit_part(&source, ordinal, |part| part.timing = None).unwrap();
    assert_eq!(model(&bytes), after);
    let text = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    assert_eq!(text, format!("{}{}", &held[..start], &held[end..]));
}

#[test]
fn an_edit_elsewhere_leaves_unread_timing_exactly_as_it_was() {
    let source = read_fixture("hostile/timing-limit-depth.bin");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| {
        part.show = Some(false);
    })
    .unwrap();
    assert_eq!(model(&bytes), wanted);
    let held = String::from_utf8(parts(&source)[&name].clone()).unwrap();
    let text = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    let block = &held[held.find("<p:timing>").unwrap()..held.find("</p:timing>").unwrap()];
    assert!(text.contains(block));
}

#[test]
fn removing_a_transition_in_a_branch_takes_the_alternate_content_with_it() {
    let source = read_fixture("generated/transition-every-effect.pptx");
    let ordinal = first_slide(&model(&source));
    let name = part_name(&model(&source), ordinal);
    let (bytes, wanted) = edit_part(&source, ordinal, |part| part.transition = None).unwrap();
    assert_eq!(model(&bytes), wanted);
    let text = String::from_utf8(parts(&bytes)[&name].clone()).unwrap();
    assert!(!text.contains("AlternateContent"), "{text}");
    f4_validates(&bytes);
}

/// A branch shared with other content is left: it is not all this edit's to remove.
#[test]
fn removing_from_alternate_content_that_holds_more_is_refused() {
    const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main""#;
    let xml = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld><mc:AlternateContent><mc:Choice Requires="p14"><p:transition/><p:timing/></mc:Choice></mc:AlternateContent></p:sld>"#
    );
    let mut part = read_part(xml.as_bytes());
    assert!(matches!(part.timing, Some(Slot::Alternate(_))));
    assert!(matches!(part.transition, Some(Slot::Alternate(_))));
    part.transition = None;
    assert_eq!(patch_part(&xml, &part).unwrap_err(), "CS-EXP-0009");
}

/// No text from a model in a refusal.
#[test]
fn nothing_from_timing_reaches_an_export_message() {
    let base = read_part(BARE.as_bytes());
    let mut part = base.clone();
    part.timing = Some(Slot::Plain(Box::new(SlideTiming {
        tn_lst: Some(vec![TimeNode::Cmd(CommandBehavior {
            cmd: Some("SECRET\u{1}".to_owned()),
            ..CommandBehavior::default()
        })]),
        bld_lst: None,
    })));
    let mut tree = XmlTree::parse(BARE.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
    let err = patch_slide_part(
        &mut tree,
        "s.xml",
        false,
        &[],
        &part,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-EXP-0001");
    assert!(!format!("{err}").contains("SECRET"));
}

// --- F4 and the validator on what was written --------------------------------------

fn f4_failures(bytes: &[u8]) -> Option<String> {
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-timing");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("edited-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let out = Command::new("python3")
        .arg("-I")
        .arg(repo().join("tools/f4/validate.py"))
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    (!out.status.success()).then(|| {
        let errors: BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.contains("Schemas validity error"))
            .map(str::to_owned)
            .collect();
        errors.into_iter().collect::<Vec<_>>().join("\n")
    })
}

fn f4_validates(bytes: &[u8]) {
    if let Some(errors) = f4_failures(bytes) {
        panic!("{errors}");
    }
}

/// Microsoft's validator on a deck this writer changed, when it is here. CI always has it.
fn sdk_passes(name: &str, bytes: &[u8]) {
    if Command::new("dotnet").arg("--version").output().is_err() {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sdk-timing");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.pptx"));
    std::fs::write(&file, bytes).unwrap();
    let out = Command::new("python3")
        .arg("-I")
        .arg(repo().join("tools/ooxml-sdk/validate.py"))
        .arg(&file)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{name}: {text}");
}
