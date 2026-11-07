//! Tier 1L: who a retained object is, what stands in for it, and that every retained part is
//! named. Nothing here may need the bytes of an object to be real, as nothing opens them.

use std::path::Path;

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::retained::{
    RetainedPart, RetainedPartKind as K, RetainedStatus, retained_report,
};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::retained::{RetainedKind, RetainedObject};
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::SlideKind;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const SLIDE: &str = "ppt/slides/slide1.xml";

fn fixture(path: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(path),
    )
    .unwrap()
}

/// The hostile deck's object, its stored bytes changed after its checksum was written.
fn damaged(path: &str) -> Vec<u8> {
    let mut bytes = fixture(path);
    let at = bytes
        .windows(4)
        .position(|w| w == [0xd0, 0xcf, 0x11, 0xe0])
        .unwrap();
    bytes[at] ^= 0xFF;
    bytes
}

fn open(path: &str) -> PresentationPackage {
    PresentationPackage::open(fixture(path), OoxmlLimits::default()).unwrap()
}

fn objects(node: &ShapeNode, out: &mut Vec<RetainedObject>) {
    out.extend(node.retained.as_deref().cloned());
    for child in &node.children {
        objects(child, out);
    }
}

fn read(path: &str) -> (Vec<RetainedObject>, Vec<RetainedObject>, Vec<String>) {
    let mut package = open(path);
    let xml = package.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&xml, SLIDE, &OoxmlLimits::default()).unwrap();
    let rels = package.relationships_of(SLIDE).to_vec();
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        SLIDE,
        &rels,
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .unwrap();
    let common = part.common.as_ref().unwrap();
    let mut found = Vec::new();
    objects(common.shape_tree.as_ref().unwrap(), &mut found);
    let report = findings
        .iter()
        .map(|f| format!("{} {} {:?}", f.kind.code(), f.element, f.attribute))
        .collect();
    (found, common.controls.clone(), report)
}

#[test]
fn an_ole_object_is_read_from_both_branches_and_linked_to_its_fallback() {
    let (found, _, report) = read("generated/area-ole.pptx");
    assert_eq!(found.len(), 4);
    let first = &found[0];
    assert_eq!(first.kind, RetainedKind::OleEmbedded);
    // `spid` is only in the choice, and the picture only in the fallback.
    assert_eq!(first.spid.as_deref(), Some("_x0000_s1026"));
    assert_eq!(first.fallback_rel_id.as_deref(), Some("rId5"));
    assert_eq!(first.display_name.as_deref(), Some("Sheet"));
    assert_eq!(first.prog_id.as_deref(), Some("Excel.Sheet.12"));
    assert_eq!(first.show_as_icon, Some(true));
    assert_eq!(first.img_w, Some(914400));
    assert_eq!(first.img_h, Some(914400));
    assert_eq!(first.rel_id.as_deref(), Some("rId2"));
    assert_eq!(found[1].rel_id.as_deref(), Some("rId3"), "a package");
    assert_eq!(found[3].kind, RetainedKind::OleLinked);
    assert!(report.iter().any(|f| f.contains("oleObj")), "{report:?}");
    assert!(report.iter().any(|f| f.contains("embed")), "{report:?}");
}

#[test]
fn a_control_is_read_with_its_fallback_and_never_more() {
    let (_, controls, report) = read("generated/area-ole.pptx");
    assert_eq!(controls.len(), 2, "one per control, not per branch");
    assert_eq!(controls[0].kind, RetainedKind::Control);
    assert_eq!(controls[0].spid.as_deref(), Some("1030"));
    assert_eq!(controls[0].rel_id.as_deref(), Some("rId6"));
    assert_eq!(controls[0].fallback_rel_id.as_deref(), Some("rId5"));
    assert_eq!(controls[1].display_name.as_deref(), Some("Plain"));
    assert!(report.iter().any(|f| f.contains("controls")), "{report:?}");
}

#[test]
fn a_model_rides_on_the_picture_drawn_in_its_place() {
    let (found, _, report) = read("generated/area-model3d.pptx");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, RetainedKind::Model3d);
    assert_eq!(found[0].rel_id.as_deref(), Some("rId2"));
    assert_eq!(found[0].fallback_rel_id.as_deref(), Some("rId3"));
    // Both links land on what they say, so nothing was reported as wrong or dangling.
    assert!(
        !report
            .iter()
            .any(|f| f.starts_with("CS-IMF-0006") || f.starts_with("CS-IMF-0007")),
        "{report:?}"
    );
    assert!(
        report.iter().any(|f| f.contains("AlternateContent")),
        "{report:?}"
    );
}

fn part<'a>(report: &'a [RetainedPart], target: &str) -> &'a RetainedPart {
    report.iter().find(|p| p.target == target).unwrap()
}

#[test]
fn the_report_names_every_retained_part() {
    let report = retained_report(&open("generated/area-ole.pptx"));
    let source = "ppt/slides/slide1.xml";
    assert_eq!(part(&report, "ppt/embeddings/oleObject1.bin").kind, K::Ole);
    assert_eq!(
        part(&report, "ppt/embeddings/oleObject1.bin").source,
        source
    );
    assert_eq!(part(&report, "ppt/embeddings/package1.bin").kind, K::Ole);
    assert_eq!(part(&report, "ppt/activeX/activeX1.xml").kind, K::Control);
    assert_eq!(part(&report, "ppt/activeX/activeX1.bin").kind, K::Control);
    assert_eq!(
        part(&report, "ppt/activeX/activeX1.bin").source,
        "ppt/activeX/activeX1.xml"
    );
    assert_eq!(part(&report, "ppt/drawings/vmlDrawing1.vml").kind, K::Vml);
    assert_eq!(part(&report, "ppt/drawings/vmlDrawing1.vml").source, source);
    let link = report
        .iter()
        .find(|p| p.status == RetainedStatus::External)
        .unwrap();
    assert_eq!(link.target, "", "a link's target is document content");
    assert_eq!(
        part(
            &retained_report(&open("generated/area-model3d.pptx")),
            "ppt/media/model3d1.glb"
        )
        .kind,
        K::Model3d
    );
    assert_eq!(
        part(
            &retained_report(&open("generated/area-vml.pptx")),
            "ppt/drawings/vmlDrawing1.vml"
        )
        .status,
        RetainedStatus::Kept
    );
}

/// `SEC-C07`: a `.pptm` opens, its macro project is never run or read, and the report
/// names it, which is what the notice shows.
#[test]
fn a_macro_deck_opens_and_the_report_names_its_project() {
    let mut package = open("generated/rr-vba.pptm");
    let deck = import_deck(&mut package, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut package,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let report = retained_report(&package);
    let project = part(&report, "ppt/vbaProject.bin");
    assert_eq!(project.kind, K::Macro);
    assert_eq!(project.source, "ppt/presentation.xml");
    assert_eq!(project.status, RetainedStatus::Kept);
    // A deck with no project has no notice.
    assert!(
        !retained_report(&open("generated/area-ole.pptx"))
            .iter()
            .any(|p| p.kind == K::Macro)
    );
}

/// A project nothing refers to is still a project in the file, and still named.
#[test]
fn a_macro_project_nothing_refers_to_is_named() {
    let mut package = open("generated/rr-vba.pptm");
    let rels = String::from_utf8(
        package
            .read_part("ppt/_rels/presentation.xml.rels")
            .unwrap(),
    )
    .unwrap();
    let (head, tail) = rels.split_once("<Relationship Id=\"rId4\"").unwrap();
    let unlinked = format!("{head}{}", tail.split_once("/>").unwrap().1);
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("ppt/_rels/presentation.xml.rels", unlinked.into_bytes());
    let saved = package.save_with(&edits).unwrap();
    let package = PresentationPackage::open(saved, OoxmlLimits::default()).unwrap();
    let report = retained_report(&package);
    let project = part(&report, "ppt/vbaProject.bin");
    assert_eq!(project.kind, K::Macro);
    assert_eq!(project.source, "");
}

#[test]
fn the_report_charges_nothing_to_the_deck_s_read_budget() {
    let package = open("generated/area-ole.pptx");
    let before = package.consumed();
    assert!(!retained_report(&package).is_empty());
    assert_eq!(package.consumed(), before);
}

#[test]
fn importing_a_whole_deck_opens_no_retained_part() {
    for (path, retained) in [
        ("generated/area-vml.pptx", "ppt/drawings/vmlDrawing1.vml"),
        ("generated/area-model3d.pptx", "ppt/media/model3d1.glb"),
        ("generated/area-ole.pptx", "ppt/embeddings/oleObject1.bin"),
        ("generated/rr-vba.pptm", "ppt/vbaProject.bin"),
    ] {
        let mut sized = open(path);
        let size = sized.read_part(retained).unwrap().len() as u64;
        // What the deck costs with the part, against the same deck with it emptied.
        let mut package = open(path);
        let deck = import_deck(&mut package, &ImportLimits::default())
            .unwrap()
            .deck;
        import_slide_parts(
            &mut package,
            &deck.presentation,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap();
        let spent = package.consumed();
        let mut again = open(path);
        again.read_part(retained).unwrap();
        // Reading the part is charged once, so a reader that had read it pays nothing more.
        let deck = import_deck(&mut again, &ImportLimits::default())
            .unwrap()
            .deck;
        import_slide_parts(
            &mut again,
            &deck.presentation,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap();
        assert_eq!(
            again.consumed(),
            spent + size,
            "{path}: {retained} was read"
        );
    }
}

#[test]
fn damaged_and_missing_parts_are_set_aside_and_the_deck_opens() {
    let path = "hostile/retained-objects-damaged.bin";
    let bytes = damaged(path);
    let package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
    let report = retained_report(&package);
    let damaged = part(&report, "ppt/embeddings/oleObject1.bin");
    assert!(
        matches!(damaged.status, RetainedStatus::SetAside { .. }),
        "{damaged:?}"
    );
    let aside = report
        .iter()
        .filter(|p| matches!(p.status, RetainedStatus::SetAside { .. }) && p.target.is_empty())
        .count();
    assert!(
        aside >= 2,
        "the escaping and the missing target: {report:?}"
    );
    // Hostile VML is not looked at: it is well formed as far as anyone here knows.
    assert_eq!(
        part(&report, "ppt/drawings/vmlDrawing1.vml").status,
        RetainedStatus::Kept
    );

    let (found, controls, findings) = read(path);
    assert_eq!(found.len(), 5);
    assert_eq!(
        found[0].img_w, None,
        "a negative or unreadable size is not kept"
    );
    assert_eq!(found[0].show_as_icon, None);
    assert_eq!(
        found[0].prog_id.as_deref(),
        Some("javascript:alert(1)"),
        "a label, never run"
    );
    assert_eq!(controls.len(), 2);
    for code in ["CS-IMF-0006", "CS-IMF-0007", "CS-IMF-0003"] {
        assert!(
            findings.iter().any(|f| f.starts_with(code)),
            "{code}: {findings:?}"
        );
    }
    // Saved unedited, the damage is still exactly what arrived.
    assert_eq!(package.save_unedited(), bytes);
}

/// The shape-wide silent drop test reads with no relationships, so a dangling `r:id` names
/// every object for it. With the deck's own, each kept element has to be named for itself.
#[test]
fn every_kept_element_of_an_object_is_named_with_its_relationships_in_place() {
    let (_, _, report) = read("generated/area-ole.pptx");
    assert!(
        !report.iter().any(|f| f.starts_with("CS-IMF-0006")),
        "{report:?}"
    );
    for name in ["oleObj", "embed", "link", "pic", "control", "controls"] {
        let named = format!(" {name} ");
        assert!(
            report.iter().any(|f| f.contains(&named)),
            "{name}: {report:?}"
        );
    }
    let (_, _, report) = read("generated/area-model3d.pptx");
    assert!(
        report.iter().any(|f| f.contains(" AlternateContent ")),
        "{report:?}"
    );
}
