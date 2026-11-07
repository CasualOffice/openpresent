//! A recovered deck read, edited and saved (`REC-04`, `REC-05`, `docs/29-RECOVERY.md`).

use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::presentation::save_deck;
use casual_slide_export::slide::save_slide_parts;
use casual_slide_export::writer::{SavePlan, WriterLimits, repair_deck};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{ImportedSlides, LazySlides, SlideLimits, import_slide_parts};
use casual_slide_model::presentation::Deck;
use casual_slide_model::properties::CoreProperties;
use casual_slide_model::slide::{SlidePart, SlideParts};
use casual_slide_ooxml::{FindingKind, OoxmlLimits, PresentationPackage, Repair, XmlTree};
use casual_slide_package::{Package, PartEdits, RETAINED_DIR};

const THREE: &str = "generated/three-slides.pptx";
const RICH: &str = "generated/slides-masters-layouts.pptx";
const STRICT: &str = "generated/slides-strict.pptx";
const SLIDE2: &str = "ppt/slides/slide2.xml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(root().join("fixtures").join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn with(deck: &[u8], part: &str, bytes: &[u8]) -> Vec<u8> {
    let mut package = Package::open(deck.to_vec(), Default::default()).unwrap();
    let mut edits = PartEdits::default();
    edits.replace(part, bytes.to_vec());
    package.rewrite(&edits).unwrap()
}

fn without(deck: &[u8], parts: &[&str]) -> Vec<u8> {
    let mut package = Package::open(deck.to_vec(), Default::default()).unwrap();
    let mut edits = PartEdits::default();
    for part in parts {
        edits.remove(part);
    }
    package.rewrite(&edits).unwrap()
}

fn part_of(deck: &[u8], part: &str) -> Vec<u8> {
    Package::open(deck.to_vec(), Default::default())
        .unwrap()
        .read_part(part)
        .unwrap()
}

fn read_all(pkg: &mut PresentationPackage) -> (Deck, ImportedSlides) {
    let deck = import_deck(pkg, &ImportLimits::default()).unwrap().deck;
    let slides = import_slide_parts(
        pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    (deck, slides)
}

fn edit_title(pkg: &mut PresentationPackage, deck: &Deck) -> Vec<u8> {
    let mut after = deck.clone();
    after
        .properties
        .core
        .get_or_insert_with(CoreProperties::default)
        .title = Some("Edited".to_owned());
    save_deck(pkg, &after, &ImportLimits::default())
        .unwrap()
        .bytes
}

fn f4_ready() -> bool {
    let has = |tool: &str, arg: &str| Command::new(tool).arg(arg).output().is_ok();
    let ready = has("python3", "--version") && has("xmllint", "--version");
    if !ready {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
    }
    ready
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("recovery-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// F4 on everything in `dir`.
fn assert_f4_passes(dir: &Path) {
    if !f4_ready() {
        return;
    }
    let done = Command::new("python3")
        .arg("-I")
        .arg(root().join("tools/f4/validate.py"))
        .arg(dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&done.stdout);
    let failing: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            let status = l.split_whitespace().next().unwrap_or_default();
            l.starts_with("      ")
                || l.contains("FAIL")
                || matches!(
                    status,
                    "invalid" | "malformed" | "refused" | "must-understand"
                )
        })
        .take(60)
        .collect();
    assert!(
        done.status.success(),
        "F4 failed:\n{}\n{}",
        failing.join("\n"),
        String::from_utf8_lossy(&done.stderr)
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn shape_count(part: &SlidePart) -> usize {
    part.shape_tree().map_or(0, |t| t.descendants().count())
}

fn nth_slide(slides: &SlideParts, n: usize) -> &SlidePart {
    slides.slide(n).unwrap()
}

/// `REC-04`: order survives, the damage is marked, and neighbours are as they were.
#[test]
fn an_unreadable_slide_is_empty_marked_and_in_its_place() {
    let source = read(THREE);
    let clean = {
        let mut pkg = open(source.clone());
        read_all(&mut pkg).1
    };
    let deck = with(&source, SLIDE2, b"\xFFthis slide is lost\x00");
    let mut pkg = open(deck.clone());
    let (model, slides) = read_all(&mut pkg);
    let structure = &slides.parts.structure;
    assert_eq!(structure.slides.len(), 3, "order survives");
    assert_eq!(slides.damaged.len(), 1, "{:?}", slides.damaged);
    let damaged = structure.entry(slides.damaged[0]).unwrap();
    assert_eq!(damaged.part, SLIDE2);
    // Empty: no shape but its tree.
    assert_eq!(shape_count(nth_slide(&slides.parts, 1)), 1);
    assert_eq!(
        nth_slide(&slides.parts, 0),
        nth_slide(&clean.parts, 0),
        "the slide before it is untouched"
    );
    assert_eq!(nth_slide(&slides.parts, 2), nth_slide(&clean.parts, 2));
    assert!(
        pkg.findings()
            .iter()
            .any(|f| f.kind == FindingKind::Repaired(Repair::SlideEmpty) && f.part == SLIDE2)
    );
    // Reading changed nothing that a save would see.
    assert_eq!(pkg.save_unedited(), deck);

    // An edit keeps its place and its bytes, and is valid.
    let edited = edit_title(&mut pkg, &model);
    let mut again = open(edited.clone());
    assert!(!again.was_repaired());
    let (_, reread) = read_all(&mut again);
    assert_eq!(reread.parts.structure.slides.len(), 3);
    let kept: Vec<String> = again
        .part_names()
        .into_iter()
        .filter(|n| n.starts_with(RETAINED_DIR))
        .collect();
    assert_eq!(kept.len(), 1);
    assert_eq!(
        again.read_part(&kept[0]).unwrap(),
        b"\xFFthis slide is lost\x00"
    );
    let dir = scratch("unreadable");
    std::fs::write(dir.join("edited.pptx"), edited).unwrap();
    assert_f4_passes(&dir);
}

/// A slide read one at a time is no different: the damage shows when it is asked for.
#[test]
fn a_lazily_read_slide_is_marked_the_same() {
    let deck = with(&read(THREE), SLIDE2, b"nothing here");
    let mut pkg = open(deck);
    let model = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let mut findings = casual_slide_import::properties::Findings::new(16);
    let structure = casual_slide_import::slide::index_deck(
        &pkg,
        &model.presentation,
        &SlideLimits::default(),
        &mut findings,
    )
    .unwrap();
    let mut lazy = LazySlides::new(structure, SlideLimits::default(), ImportLimits::default());
    assert!(!lazy.slide(&mut pkg, 0).unwrap().damaged);
    assert!(lazy.slide(&mut pkg, 1).unwrap().damaged);
    assert!(!lazy.slide(&mut pkg, 2).unwrap().damaged);
}

/// What could be trusted is shown: the shape holding the damage goes, its neighbours stay.
#[test]
fn a_partly_read_slide_keeps_every_other_shape() {
    let source = read(RICH);
    let (slide_name, clean_count) = {
        let mut pkg = open(source.clone());
        let (_, slides) = read_all(&mut pkg);
        // The slide with the most shapes.
        let mut best = (String::new(), 0);
        for ordinal in &slides.parts.structure.slides {
            let name = slides.parts.structure.entry(*ordinal).unwrap().part.clone();
            let count = shape_count(slides.parts.get(*ordinal).unwrap());
            if count > best.1 {
                best = (name, count);
            }
        }
        best
    };
    assert!(clean_count >= 3, "{clean_count}");
    let xml = String::from_utf8(part_of(&source, &slide_name)).unwrap();
    // A broken attribute inside the first shape.
    let at = xml.find("<p:sp>").unwrap();
    let broken = format!("{}<p:sp broken=1>{}", &xml[..at], &xml[at + 6..]);
    let deck = with(&source, &slide_name, broken.as_bytes());
    let mut pkg = open(deck);
    let (model, slides) = read_all(&mut pkg);
    let ordinal = *slides
        .parts
        .structure
        .slides
        .iter()
        .find(|o| slides.parts.structure.entry(**o).unwrap().part == slide_name)
        .unwrap();
    assert!(slides.damaged.contains(&ordinal));
    let count = shape_count(slides.parts.get(ordinal).unwrap());
    assert!(count > 1 && count < clean_count, "{count} of {clean_count}");
    let edited = edit_title(&mut pkg, &model);
    let dir = scratch("partly");
    std::fs::write(dir.join("edited.pptx"), edited).unwrap();
    assert_f4_passes(&dir);
}

/// `REC-05`: the user can edit a part that was damaged, and what is written is valid.
#[test]
fn a_damaged_slide_can_be_edited() {
    let source = read(RICH);
    let xml = String::from_utf8(part_of(&source, SLIDE2)).unwrap();
    // Cut off in the middle of the part, somewhere after the shape tree begins.
    let cut = xml.rfind("</p:sp>").map_or(xml.len() / 2, |i| i + 7);
    let deck = with(&source, SLIDE2, &xml.as_bytes()[..cut]);
    let mut pkg = open(deck);
    let (_, slides) = read_all(&mut pkg);
    let mut wanted = slides.parts.clone();
    let ordinal = wanted.structure.slide(1).unwrap();
    let structure = wanted.structure.clone();
    let part = wanted.get_mut(ordinal).unwrap();
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .filter(|n| {
            n.non_visual
                .as_ref()
                .and_then(|v| v.name.as_deref())
                .is_some()
        })
        .last()
        .unwrap()
        .id;
    part.node_mut(id).unwrap().non_visual.as_mut().unwrap().name =
        Some("Renamed after repair".to_owned());
    let part = wanted.get(ordinal).unwrap();
    let outcome = save_slide_parts(
        &mut pkg,
        &structure,
        &[part],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert!(outcome.changed_parts.iter().any(|p| p == SLIDE2));
    let mut again =
        PresentationPackage::open_strict(outcome.bytes.clone(), OoxmlLimits::default()).unwrap();
    let text = String::from_utf8(again.read_part(SLIDE2).unwrap()).unwrap();
    assert!(text.contains("Renamed after repair"), "{text}");
    let dir = scratch("edited-slide");
    std::fs::write(dir.join("edited.pptx"), outcome.bytes).unwrap();
    assert_f4_passes(&dir);
}

/// `REC-04`: what is missing is made in memory, and reaches a file only through an edit.
#[test]
fn a_missing_layout_master_or_theme_is_made_and_written_only_on_edit() {
    for (fixture, form) in [(RICH, "transitional"), (STRICT, "strict")] {
        let source = read(fixture);
        for gone in [
            "ppt/slideLayouts/slideLayout1.xml",
            "ppt/slideMasters/slideMaster1.xml",
            "ppt/theme/theme1.xml",
        ] {
            let deck = without(&source, &[gone]);
            let mut pkg = open(deck.clone());
            assert!(pkg.was_repaired(), "{fixture} {gone}");
            // In memory, and reading it finds it.
            assert!(pkg.read_part(gone).is_ok(), "{fixture} {gone}");
            let (model, slides) = read_all(&mut pkg);
            // Slides stay in order, each with the chain it needs.
            for ordinal in &slides.parts.structure.slides {
                let (layout, master) = slides.parts.chain(*ordinal);
                assert!(layout.is_some(), "{fixture} {gone}");
                assert!(master.is_some(), "{fixture} {gone}");
            }
            assert_eq!(pkg.save_unedited(), deck, "{fixture} {gone}");
            let edited = edit_title(&mut pkg, &model);
            let names = Package::open(edited.clone(), Default::default())
                .unwrap()
                .entry_names();
            assert!(
                names.iter().any(|n| n == gone),
                "{fixture} {gone}: {names:?}"
            );
            let dir = scratch(&format!("missing-{form}-{}", gone.replace('/', "_")));
            std::fs::write(dir.join("edited.pptx"), edited).unwrap();
            assert_f4_passes(&dir);
        }
    }
}

/// A slide with no layout at all, and a deck with no masters: the nearest is used, or one made.
#[test]
fn a_slide_with_no_layout_takes_the_nearest() {
    let source = read(RICH);
    let rels = "ppt/slides/_rels/slide1.xml.rels";
    let deck = without(&source, &[rels]);
    let mut pkg = open(deck);
    let (model, slides) = read_all(&mut pkg);
    let first = slides.parts.structure.slide(0).unwrap();
    assert!(slides.parts.chain(first).0.is_some());
    assert!(
        pkg.findings()
            .iter()
            .any(|f| f.kind == FindingKind::Repaired(Repair::LayoutSupplied))
    );
    let edited = edit_title(&mut pkg, &model);
    let dir = scratch("nearest");
    std::fs::write(dir.join("edited.pptx"), edited).unwrap();
    assert_f4_passes(&dir);
}

/// `REC-05`, every hostile fixture: open, read, edit, and F4 passes what comes out.
#[test]
fn every_hostile_fixture_can_be_edited_into_a_valid_file() {
    let manifest = std::fs::read_to_string(root().join("fixtures/manifest.tsv")).unwrap();
    let dir = scratch("hostile");
    let mut edited_count = 0;
    for line in manifest.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        // Those lowered limits make refuse are written under their own limits elsewhere; the
        // others are the attacks a default deployment meets.
        if !cols[2].starts_with("recover:") || cols.len() > 3 {
            continue;
        }
        let bytes = std::fs::read(root().join("fixtures").join(cols[0])).unwrap();
        let mut pkg = open(bytes);
        let (model, _) = read_all(&mut pkg);
        let edited = edit_title(&mut pkg, &model);
        let name = cols[0].replace('/', "_").replace(".bin", ".pptx");
        std::fs::write(dir.join(name), edited).unwrap();
        edited_count += 1;
    }
    assert!(edited_count >= 25, "{edited_count}");
    assert_f4_passes(&dir);
}

/// `REC-05`: an explicit repair, with nothing edited, is a clean file by the same preflight.
#[test]
fn an_explicit_repair_writes_a_clean_file() {
    let deck = without(&read(RICH), &["[Content_Types].xml", "_rels/.rels"]);
    let mut pkg = open(deck.clone());
    let outcome = repair_deck(&mut pkg, WriterLimits::default()).unwrap();
    assert_ne!(outcome.bytes, deck);
    assert_eq!(pkg.save_unedited(), deck);
    PresentationPackage::open_strict(outcome.bytes.clone(), OoxmlLimits::default()).unwrap();
    let dir = scratch("explicit");
    std::fs::write(dir.join("repaired.pptx"), outcome.bytes).unwrap();
    assert_f4_passes(&dir);
    // A healthy deck has nothing to repair, and comes back as it came.
    let healthy = read(RICH);
    let mut pkg = open(healthy.clone());
    assert_eq!(
        repair_deck(&mut pkg, WriterLimits::default())
            .unwrap()
            .bytes,
        healthy
    );
}

/// A plan that changes nothing is an unedited save, so the damage comes back.
#[test]
fn a_plan_with_no_edit_returns_the_damage() {
    let deck = with(&read(THREE), SLIDE2, b"<broken");
    let mut pkg = open(deck.clone());
    let outcome = SavePlan::new().save(&mut pkg).unwrap();
    assert_eq!(outcome.bytes, deck);
}

/// A hostile size can only make this slow if something is sized from it: a limit stops the
/// reading of one slide, and the others open.
#[test]
fn a_limit_stops_one_part_and_the_deck_opens() {
    let bytes = read("hostile/slide-limit-shapes.bin");
    let mut pkg = open(bytes);
    let (_, slides) = read_all(&mut pkg);
    assert_eq!(slides.damaged.len(), 1);
    assert!(!slides.parts.structure.slides.is_empty());
}

/// What is read from a repaired part is one reading: it parses, whichever reader looks.
#[test]
fn a_repaired_part_is_read_alike_by_every_reader() {
    let deck = with(&read(RICH), SLIDE2, b"<p:sld><p:cSld><p:spTree>");
    let mut pkg = open(deck);
    let bytes = pkg.read_xml_part(SLIDE2).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE2, &OoxmlLimits::default()).unwrap();
    assert_eq!(tree.local_name(tree.root()), "sld");
    assert_eq!(pkg.read_part(SLIDE2).unwrap(), bytes);
}

/// `FUZZ-02`, found by `deck_recover`: one byte of a central directory record's compressed
/// size changed (0x152 to 0x2b52). The deck opens; an edit to it must write a clean file,
/// not refuse with `INV-PN-10`.
#[test]
fn a_central_directory_size_that_disagrees_with_its_local_header_still_saves() {
    let mut deck = read(THREE);
    let at = 0xc030 % deck.len();
    deck[at] = 0x2b;
    let mut pkg = open(deck);
    let (d, _) = read_all(&mut pkg);
    let edited = edit_title(&mut pkg, &d);
    PresentationPackage::open_strict(edited, OoxmlLimits::default()).unwrap();
}

/// `FUZZ-03`, the second `deck_recover` input: one byte of a central directory name made
/// non-ASCII, so the directory names an entry its local header does not. An edit saves.
#[test]
fn a_central_name_that_disagrees_with_its_local_header_still_saves() {
    let mut deck = read(STRICT);
    let at = 0x6f25 % deck.len();
    deck[at] = 0xc9;
    let mut pkg = open(deck);
    let (d, _) = read_all(&mut pkg);
    let edited = edit_title(&mut pkg, &d);
    PresentationPackage::open_strict(edited, OoxmlLimits::default()).unwrap();
}

/// `REC-07`: a deck whose only package relationship to its main part has a spoofed type opens
/// by finding the main part by name (`CS-REC-0203`). An edit writes the repaired package, so
/// the output must state the main part with the real relationship type, or no consumer that
/// follows relationships, Microsoft's validator among them, can find it.
#[test]
fn an_edited_deck_found_by_name_states_its_main_part() {
    let bytes = read("hostile/spoofed-office-document-type.bin");
    let mut pkg = open(bytes);
    let (model, _) = read_all(&mut pkg);
    let edited = edit_title(&mut pkg, &model);
    let rels = String::from_utf8(part_of(&edited, "_rels/.rels")).unwrap();
    assert!(
        rels.contains(
            "Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\""
        ),
        "{rels}"
    );
    // One relationship to the main part, not the spoofed one beside it: a part reached by two
    // types is a package Microsoft's validator refuses.
    assert_eq!(
        rels.matches("Target=\"ppt/presentation.xml\"").count(),
        1,
        "{rels}"
    );
    assert!(!rels.contains("example.com/evil"), "{rels}");
}
