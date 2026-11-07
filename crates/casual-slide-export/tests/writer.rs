//! The one writer (`T1N-01` to `T1N-04`): a save plan applied atomically, with what
//! every part other than those named keeps.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::slide::save_slide_parts;
use casual_slide_export::theme::set_theme_name;
use casual_slide_export::writer::{Disposition, Op, SavePlan, SlideRef, WriterLimits, check_zip};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::presentation::Percentage;
use casual_slide_model::properties::CoreProperties;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const THREE: &str = "generated/three-slides.pptx";
const THEME: &str = "ppt/theme/theme1.xml";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: &[u8]) -> PresentationPackage {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap()
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes);
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

/// Validate with F4, as `tests/slides.rs` does.
fn f4(bytes: &[u8]) {
    let have = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    if !have("python3", "--version") || !have("xmllint", "--version") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-writer");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("w-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The theme's bytes with a new name, the way a caller gets them from `theme.rs`.
fn renamed_theme(pkg: &mut PresentationPackage, name: &str) -> Vec<u8> {
    let bytes = pkg.read_part(THEME).unwrap();
    let mut tree = XmlTree::parse(&bytes, THEME, &OoxmlLimits::default()).unwrap();
    assert!(set_theme_name(&mut tree, name).unwrap());
    tree.to_bytes()
}

#[test]
fn one_save_combines_properties_theme_and_slides() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.presentation.server_zoom = Some(Percentage(50_000));
    deck.properties.core = Some(CoreProperties {
        title: Some("Combined".to_owned()),
        ..CoreProperties::default()
    });
    let slides = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let second = slides.parts.structure.slides[1];
    let mut edited = slides.parts.get(second).unwrap().clone();
    edited.show_master_shapes = Some(false);
    let wanted_parts = [&edited];
    let theme = renamed_theme(&mut pkg, "Combined theme");
    let limits = SlideLimits::default();

    let outcome = SavePlan::new()
        .deck(&deck)
        .slide_parts(&slides.parts.structure, &wanted_parts, &limits)
        .replace_part(THEME, theme)
        .op(Op::AddSlide {
            position: None,
            layout: None,
        })
        .op(Op::DuplicateSlide {
            slide: SlideRef::Index(1),
        })
        .op(Op::DeleteSlide {
            slide: SlideRef::Index(0),
        })
        .save(&mut pkg)
        .unwrap();

    let after = parts(&outcome.bytes);
    let text = |name: &str| String::from_utf8(after[name].clone()).unwrap();
    assert!(
        text("ppt/presentation.xml").contains("serverZoom=\"50000\""),
        "the deck's patch"
    );
    assert!(text("docProps/core.xml").contains("Combined"), "properties");
    assert!(text(THEME).contains("name=\"Combined theme\""), "the theme");
    assert!(
        text("ppt/slides/slide2.xml").contains("showMasterSp=\"0\""),
        "the slide's patch"
    );
    assert!(!after.contains_key("ppt/slides/slide1.xml"), "the delete");
    assert!(after.contains_key("ppt/slides/slide4.xml"), "the add");
    assert!(after.contains_key("ppt/slides/slide5.xml"), "the duplicate");
    assert_eq!(
        after["ppt/slides/slide5.xml"], after["ppt/slides/slide2.xml"],
        "a duplicate is a byte copy"
    );

    let model = import_deck(&mut open(&outcome.bytes), &ImportLimits::default())
        .unwrap()
        .deck;
    assert_eq!(model.presentation.slides.as_ref().unwrap().len(), 4);
    assert_eq!(model.presentation.server_zoom, Some(Percentage(50_000)));
    f4(&outcome.bytes);
}

/// `T1B-02`, by way of `T1N-05`: a deck with counts and titles in `app.xml`.
const RR_APP: &str = "generated/rr-app.pptx";
const APP: &str = "docProps/app.xml";

fn app_of(bytes: &[u8]) -> String {
    String::from_utf8(parts(bytes)[APP].clone()).unwrap()
}

fn add_slide() -> Op {
    Op::AddSlide {
        position: None,
        layout: None,
    }
}

/// A model patch that rewrites only the main part states nothing about `app.xml`, so the
/// counts and titles still follow the slides the same save added and deleted.
#[test]
fn app_counts_follow_the_slides_when_a_deck_patch_also_rewrote_the_main_part() {
    let mut pkg = open(&read(RR_APP));
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.presentation.server_zoom = Some(Percentage(50_000));
    let outcome = SavePlan::new()
        .deck(&deck)
        .op(add_slide())
        .op(add_slide())
        .save(&mut pkg)
        .unwrap();
    let app = app_of(&outcome.bytes);
    assert!(app.contains("<Slides>5</Slides>"), "{app}");
    assert!(app.contains("<vt:i4>5</vt:i4>"), "{app}");
    assert!(app.contains("size=\"6\""), "{app}");
}

/// Even a patch that states `app.xml` does not get to leave the count behind slides this
/// same save changed: that would write a file whose `Slides` is known to be wrong.
#[test]
fn app_counts_win_over_a_stated_count_only_when_the_slides_changed() {
    let mut pkg = open(&read(RR_APP));
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.properties.app.as_mut().unwrap().company = Some("Acme".to_owned());
    let added = SavePlan::new()
        .deck(&deck)
        .op(add_slide())
        .save(&mut pkg)
        .unwrap();
    let app = app_of(&added.bytes);
    assert!(app.contains("<Slides>4</Slides>"), "{app}");
    assert!(app.contains("<Company>Acme</Company>"), "{app}");

    let mut pkg = open(&read(RR_APP));
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.properties.app.as_mut().unwrap().slides = Some(7);
    let stated = SavePlan::new().deck(&deck).save(&mut pkg).unwrap();
    let app = app_of(&stated.bytes);
    assert!(
        app.contains("<Slides>7</Slides>"),
        "what it states stands: {app}"
    );
}

/// A patch alone that changes the slide list answers for the count too: what it says reads
/// back. It is reported, not silent. Whatever moves the slides after it is followed.
#[test]
fn a_deck_patch_that_changes_the_list_answers_for_the_count_and_is_reported() {
    let mut pkg = open(&read(RR_APP));
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.presentation.slides.as_mut().unwrap().remove(2);
    let alone = SavePlan::new().deck(&deck).save(&mut pkg).unwrap();
    let app = app_of(&alone.bytes);
    assert!(app.contains("<Slides>3</Slides>"), "{app}");
    assert!(
        alone.report.iter().any(|e| e.rule == "INV-AP-01"),
        "{:?}",
        alone.report
    );

    let mut pkg = open(&read(RR_APP));
    let then_added = SavePlan::new()
        .deck(&deck)
        .op(add_slide())
        .save(&mut pkg)
        .unwrap();
    let model = import_deck(&mut open(&then_added.bytes), &ImportLimits::default())
        .unwrap()
        .deck;
    assert_eq!(model.presentation.slides.as_ref().unwrap().len(), 3);
    let app = app_of(&then_added.bytes);
    assert!(app.contains("<Slides>3</Slides>"), "{app}");
}

#[test]
fn app_notes_count_follows_adding_and_removing_notes() {
    let mut pkg = open(&read(RR_APP));
    let added = SavePlan::new()
        .op(Op::AddNotes {
            slide: SlideRef::Index(2),
            text: "A note".to_owned(),
        })
        .save(&mut pkg)
        .unwrap();
    let app = app_of(&added.bytes);
    assert!(app.contains("<Notes>2</Notes>"), "{app}");
    let mut pkg = open(&added.bytes);
    let removed = SavePlan::new()
        .op(Op::RemoveNotes {
            slide: SlideRef::Index(2),
        })
        .save(&mut pkg)
        .unwrap();
    let app = app_of(&removed.bytes);
    assert!(app.contains("<Notes>1</Notes>"), "{app}");
}

#[test]
fn app_hidden_count_follows_a_slide_patch_alongside_a_deck_patch() {
    let mut pkg = open(&read(RR_APP));
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    deck.presentation.server_zoom = Some(Percentage(50_000));
    let slides = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let structure = &slides.parts.structure;
    let shown = structure
        .slides
        .iter()
        .map(|&id| slides.parts.get(id).unwrap())
        .find(|p| p.show != Some(false))
        .unwrap();
    let mut edited = shown.clone();
    edited.show = Some(false);
    let wanted = [&edited];
    let limits = SlideLimits::default();
    let outcome = SavePlan::new()
        .deck(&deck)
        .slide_parts(structure, &wanted, &limits)
        .save(&mut pkg)
        .unwrap();
    let app = app_of(&outcome.bytes);
    assert!(app.contains("<HiddenSlides>2</HiddenSlides>"), "{app}");
}

#[test]
fn a_plan_that_fails_part_way_writes_nothing_and_a_second_try_works() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let err = SavePlan::new()
        .op(Op::AddSlide {
            position: None,
            layout: None,
        })
        .op(Op::DeleteSlide {
            slide: SlideRef::Index(99),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0017");
    assert_eq!(pkg.save_unedited(), source, "the session is intact");
    let good = SavePlan::new()
        .op(Op::AddSlide {
            position: None,
            layout: None,
        })
        .save(&mut pkg)
        .unwrap();
    assert!(parts(&good.bytes).contains_key("ppt/slides/slide4.xml"));
}

/// Found by the fuzz target: a new slide's neighbour going later in the plan.
#[test]
fn a_new_slide_stays_in_its_section_when_the_slide_before_it_goes() {
    let source = read("generated/rr-sections.pptx");
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::DuplicateSlide {
            slide: SlideRef::Index(2),
        })
        .op(Op::DeleteSlide {
            slide: SlideRef::Index(2),
        })
        .save(&mut pkg)
        .unwrap();
    assert_eq!(outcome.new_slides, ["ppt/slides/slide4.xml"]);
}

/// Found by the fuzz target: a part this plan added, and this plan then took away.
#[test]
fn a_part_added_and_removed_in_one_plan_is_not_written() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::AddLayout {
            master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
            copy_of: "ppt/slideLayouts/slideLayout1.xml".to_owned(),
        })
        .op(Op::RemoveLayout {
            layout: "ppt/slideLayouts/slideLayout2.xml".to_owned(),
        })
        .op(Op::DuplicateSlide {
            slide: SlideRef::Index(0),
        })
        .op(Op::DeleteSlide {
            slide: SlideRef::Index(1),
        })
        .op(Op::AddNotes {
            slide: SlideRef::Index(0),
            text: "Gone".to_owned(),
        })
        .op(Op::RemoveNotes {
            slide: SlideRef::Index(0),
        })
        .save(&mut pkg)
        .unwrap();
    let after = parts(&outcome.bytes);
    assert!(!after.contains_key("ppt/slideLayouts/slideLayout2.xml"));
    assert!(!after.contains_key("ppt/slides/slide4.xml"));
    assert!(!after.contains_key("ppt/slides/_rels/slide4.xml.rels"));
    assert!(!after.contains_key("ppt/notesSlides/notesSlide1.xml"));
    assert!(
        after.contains_key("ppt/notesMasters/notesMaster1.xml"),
        "the master stays once made"
    );
    let types = String::from_utf8(after["[Content_Types].xml"].clone()).unwrap();
    assert!(!types.contains("slide4.xml"), "{types}");
    f4(&outcome.bytes);
}

#[test]
fn a_plan_with_nothing_in_it_returns_the_input_bytes() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let outcome = SavePlan::new().save(&mut pkg).unwrap();
    assert_eq!(outcome.bytes, source);
    assert!(outcome.changed_parts.is_empty());
    assert!(
        outcome
            .dispositions
            .values()
            .all(|d| *d == Disposition::UnchangedCopy)
    );
}

#[test]
fn undoing_within_a_plan_returns_the_input_bytes_when_nothing_changed() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let theme = pkg.read_part(THEME).unwrap();
    let outcome = SavePlan::new()
        .replace_part(THEME, theme)
        .save(&mut pkg)
        .unwrap();
    assert_eq!(outcome.bytes, source);
}

/// Every part not named keeps its bytes, checked on the entries' compressed form and
/// separately on what they decompress to.
#[test]
fn every_other_part_stays_byte_identical() {
    let source = read("generated/slides-masters-layouts.pptx");
    let mut pkg = open(&source);
    let outcome = SavePlan::new()
        .op(Op::AddSlide {
            position: Some(0),
            layout: None,
        })
        .save(&mut pkg)
        .unwrap();
    let (old, new) = (parts(&source), parts(&outcome.bytes));
    let changed: BTreeSet<&String> = outcome.changed_parts.iter().collect();
    assert_eq!(
        outcome.changed_parts,
        vec![
            "[Content_Types].xml".to_owned(),
            "ppt/_rels/presentation.xml.rels".to_owned(),
            "ppt/presentation.xml".to_owned(),
            outcome.new_slides[0].clone(),
            format!(
                "ppt/slides/_rels/{}.rels",
                outcome.new_slides[0].rsplit('/').next().unwrap()
            ),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
    );
    for (name, bytes) in &old {
        if !changed.contains(name) {
            assert_eq!(new.get(name), Some(bytes), "{name}");
        }
    }
    check_zip(
        &source,
        &outcome.bytes,
        &outcome.changed_parts.iter().cloned().collect(),
    )
    .unwrap();
    assert_eq!(
        outcome.dispositions["ppt/presentation.xml"],
        Disposition::Merge
    );
    assert_eq!(
        outcome.dispositions[&outcome.new_slides[0]],
        Disposition::Regenerate
    );
    assert_eq!(
        outcome.dispositions["ppt/theme/theme1.xml"],
        Disposition::UnchangedCopy
    );
}

/// The check this leans on sees a retained entry change.
#[test]
fn a_retained_part_that_changes_is_caught_after_writing() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let bytes = pkg
        .save_with(&{
            let mut edits = casual_slide_package::PartEdits::default();
            edits.replace(THEME, b"<a:theme/>".to_vec());
            edits
        })
        .unwrap();
    let err = check_zip(&source, &bytes, &BTreeSet::new()).unwrap_err();
    assert!(err.names_rule("F2"), "{err}");
    check_zip(&source, &bytes, &[THEME.to_owned()].into_iter().collect()).unwrap();
}

#[test]
fn save_deck_and_save_slide_parts_are_this_writer() {
    // The second reaches preflight: a hand-built package with a section naming a slide
    // has nothing to do with it, and a patch of a slide part still passes through.
    let source = read(THREE);
    let mut pkg = open(&source);
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let limits = SlideLimits::default();
    let slides = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &limits,
        &ImportLimits::default(),
    )
    .unwrap();
    let first = slides.parts.structure.slides[0];
    let mut part = slides.parts.get(first).unwrap().clone();
    part.show_master_shapes = Some(false);
    let outcome = save_slide_parts(
        &mut pkg,
        &slides.parts.structure,
        &[&part],
        &limits,
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(
        outcome.changed_parts,
        vec!["ppt/slides/slide1.xml".to_owned()]
    );
}

#[test]
fn a_plan_over_the_limit_is_refused() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let limits = WriterLimits {
        max_plan_ops: 1,
        ..WriterLimits::default()
    };
    let add = Op::AddSlide {
        position: None,
        layout: None,
    };
    let err = SavePlan::new()
        .with_limits(ImportLimits::default(), limits)
        .op(add.clone())
        .op(add)
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0018");
}

#[test]
fn a_clone_over_its_limit_is_refused_before_it_is_copied() {
    let source = read("generated/image.pptx");
    let mut pkg = open(&source);
    let too_few = WriterLimits {
        max_clone_parts: 0,
        ..WriterLimits::default()
    };
    let err = SavePlan::new()
        .with_limits(ImportLimits::default(), too_few)
        .op(Op::DuplicateSlide {
            slide: SlideRef::Index(0),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0018");
    let too_small = WriterLimits {
        max_clone_bytes: 1,
        ..WriterLimits::default()
    };
    let err = SavePlan::new()
        .with_limits(ImportLimits::default(), too_small)
        .op(Op::DuplicateSlide {
            slide: SlideRef::Index(0),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0018");
}

#[test]
fn media_over_its_limit_is_refused() {
    let source = read(THREE);
    let mut pkg = open(&source);
    let limits = WriterLimits {
        max_media_bytes: 2,
        ..WriterLimits::default()
    };
    let err = SavePlan::new()
        .with_limits(ImportLimits::default(), limits)
        .op(Op::AddPicture {
            slide: SlideRef::Index(0),
            bytes: vec![1, 2, 3],
            extension: "png".to_owned(),
            content_type: "image/png".to_owned(),
        })
        .save(&mut pkg)
        .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0018");
}

/// `WR-11`: something new in a Strict deck takes the Strict spelling of an ECMA namespace,
/// and the released namespace of a Microsoft one. `a14:` lists the pre-release Office 2010
/// namespace second, for reading only; Strict took the second entry of every list, so a new
/// `a14` element was declared in the namespace no released PowerPoint reads.
#[test]
fn a_new_element_in_a_strict_deck_takes_the_released_namespace() {
    use casual_slide_export::properties::Writer;
    use casual_slide_import::drawing::A14;
    use casual_slide_import::properties::{Elem, ns};

    const STRICT_PML: &str = "http://purl.oclc.org/ooxml/presentationml/main";
    let xml = format!("<p:sld xmlns:p=\"{STRICT_PML}\"/>");
    let mut tree = XmlTree::parse(xml.as_bytes(), "s", &OoxmlLimits::default()).unwrap();
    let root = Elem::root(&tree, "s").unwrap();
    let mut w = Writer {
        tree: &mut tree,
        part: "s",
        strict: true,
        known_rels: None,
        changes: 0,
    };
    assert_eq!(
        w.uri(A14),
        "http://schemas.microsoft.com/office/drawing/2010/main"
    );
    assert_eq!(w.uri(ns::DML), "http://purl.oclc.org/ooxml/drawingml/main");
    let kid = w.new_child(&root, A14, "a14", "hiddenFill").unwrap();
    let a = w.new_child(&root, ns::DML, "a", "ext").unwrap();
    for id in [kid.id, a.id] {
        tree.insert_child(root.id, None, id);
    }
    let out = String::from_utf8(tree.to_bytes()).unwrap();
    assert!(
        out.contains("xmlns:a14=\"http://schemas.microsoft.com/office/drawing/2010/main\""),
        "{out}"
    );
    assert!(!out.contains("2007/7/7"), "{out}");
    assert!(
        out.contains("xmlns:a=\"http://purl.oclc.org/ooxml/drawingml/main\""),
        "{out}"
    );
    // Transitional is unchanged.
    let mut tree = XmlTree::parse(b"<x/>", "t", &OoxmlLimits::default()).unwrap();
    let w = Writer {
        tree: &mut tree,
        part: "t",
        strict: false,
        known_rels: None,
        changes: 0,
    };
    assert_eq!(
        w.uri(A14),
        "http://schemas.microsoft.com/office/drawing/2010/main"
    );
    assert_eq!(
        w.uri(ns::DML),
        "http://schemas.openxmlformats.org/drawingml/2006/main"
    );
}
