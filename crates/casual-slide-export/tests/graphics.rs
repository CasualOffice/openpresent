//! Tier 1K, against decks: SmartArt, equations and ink read, written and held.
//!
//! The row walk reads `docs/spec/checklist.tsv`, so a row added to DGM, MATH or INK must
//! be read and written here or be named. Each deck's parts go through a forced rewrite: the
//! part is emptied and written again from the model alone, since an unedited save proves
//! nothing about the writer (`WR-01`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::diagram::{patch_diagram_part, save_diagram};
use casual_slide_export::ink::{patch_ink_part, save_ink_part};
use casual_slide_export::math::patch_math_part;
use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::diagram::{
    GraphicLimits, import_diagram, import_diagram_root, import_slide_diagrams,
};
use casual_slide_import::ink::{import_ink_part, import_ink_root, import_slide_ink};
use casual_slide_import::math::import_math_root;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part};
use casual_slide_model::diagram::schema as dgm;
use casual_slide_model::diagram::{Diagram, DiagramRole, Inconsistency};
use casual_slide_model::ink::schema as ink;
use casual_slide_model::math::MathItem;
use casual_slide_model::math::schema as omml;
use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, Relationship, XmlTree};
use casual_slide_package::PartEdits;

const SLIDE: &str = "ppt/slides/slide1.xml";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(path: &str) -> Vec<u8> {
    std::fs::read(repo().join("fixtures").join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn limits() -> GraphicLimits {
    GraphicLimits::default()
}

fn import() -> ImportLimits {
    ImportLimits::default()
}

fn slide(pkg: &mut PresentationPackage, part: &str) -> SlidePart {
    let bytes = pkg.read_part(part).unwrap();
    let tree = XmlTree::parse(&bytes, part, &OoxmlLimits::default()).unwrap();
    let rels = pkg.relationships_of(part).to_vec();
    read_slide(&tree, part, &rels)
}

fn read_slide(tree: &XmlTree, part: &str, rels: &[Relationship]) -> SlidePart {
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree,
        part,
        rels,
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .unwrap()
}

/// Every diagram frame of a deck: the slide-like part holding it, and its relationship ids.
fn frames(pkg: &mut PresentationPackage) -> Vec<(String, dgm::RelIds)> {
    let mut out = Vec::new();
    for name in pkg.part_names() {
        if !name.starts_with("ppt/slides/slide") || !name.ends_with(".xml") {
            continue;
        }
        let part = slide(pkg, &name);
        for node in part
            .shape_tree()
            .into_iter()
            .flat_map(ShapeNode::descendants)
        {
            if let Some(ids) = node.diagram.as_deref() {
                out.push((name.clone(), ids.clone()));
            }
        }
    }
    out
}

fn diagram_of(pkg: &mut PresentationPackage, part: &str, ids: &dgm::RelIds) -> (Diagram, Findings) {
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let d = import_diagram(pkg, part, ids, &limits(), &import, &mut findings);
    (d, findings)
}

/// The first frame's diagram.
fn first_diagram(bytes: &[u8]) -> (PresentationPackage, String, dgm::RelIds, Diagram) {
    let mut pkg = open(bytes.to_vec());
    let (part, ids) = frames(&mut pkg).remove(0);
    let (d, _) = diagram_of(&mut pkg, &part, &ids);
    (pkg, part, ids, d)
}

fn part_tree(pkg: &mut PresentationPackage, name: &str) -> XmlTree {
    let bytes = pkg.read_part(name).unwrap();
    XmlTree::parse(&bytes, name, &OoxmlLimits::default()).unwrap()
}

/// A tree as names, attributes (declarations aside) and text, order kept.
fn canonical(tree: &XmlTree, id: ElementId, out: &mut String) {
    out.push('<');
    out.push_str(tree.qualified_name(id));
    let mut attrs: Vec<(String, String)> = tree
        .declared_attributes(id)
        .into_iter()
        .filter(|(k, _)| k != "xmlns" && !k.starts_with("xmlns:"))
        .collect();
    attrs.sort();
    for (k, v) in attrs {
        out.push_str(&format!(" {k}={v:?}"));
    }
    out.push('>');
    let kids = tree.child_elements(id);
    if kids.is_empty()
        && let Some(text) = tree.own_text(id)
    {
        out.push_str(&text);
    }
    for kid in kids {
        canonical(tree, kid, out);
    }
    out.push('/');
}

fn canon(tree: &XmlTree) -> String {
    let mut out = String::new();
    canonical(tree, tree.root(), &mut out);
    out
}

/// The same part with its root emptied: its namespaces stay, nothing else does.
fn emptied(tree: &XmlTree) -> XmlTree {
    let mut out = tree.clone();
    let root = out.root();
    for kid in out.child_elements(root) {
        out.remove_child(root, kid);
    }
    let attrs = out.declared_attributes(root);
    for (k, _) in attrs {
        if k != "xmlns" && !k.starts_with("xmlns:") {
            out.remove_attribute(root, &k);
        }
    }
    out
}

fn read_dgm_root(tree: &XmlTree, part: &str, rels: &[Relationship]) -> Option<dgm::Root> {
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    import_diagram_root(tree, part, rels, &limits(), &import, &mut findings).unwrap()
}

fn read_math_root(tree: &XmlTree, part: &str) -> Option<omml::Root> {
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    import_math_root(tree, part, &[], &limits(), &import, &mut findings).unwrap()
}

fn read_ink_root(tree: &XmlTree, part: &str) -> Option<ink::Root> {
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    import_ink_root(tree, part, &[], &limits(), &import, &mut findings).unwrap()
}

/// One kind of part, for walks that do not care which it is.
#[derive(Clone, Copy)]
enum Kind {
    Diagram,
    Math,
    Ink,
}

#[derive(PartialEq, Debug)]
enum AnyRoot {
    Dgm(Box<dgm::Root>),
    Math(Box<omml::Root>),
    Ink(Box<ink::Root>),
}

fn read_any(kind: Kind, tree: &XmlTree, part: &str, rels: &[Relationship]) -> Option<AnyRoot> {
    Some(match kind {
        Kind::Diagram => AnyRoot::Dgm(Box::new(read_dgm_root(tree, part, rels)?)),
        Kind::Math => AnyRoot::Math(Box::new(read_math_root(tree, part)?)),
        Kind::Ink => AnyRoot::Ink(Box::new(read_ink_root(tree, part)?)),
    })
}

fn patch_any(
    tree: &mut XmlTree,
    part: &str,
    rels: &[Relationship],
    to: &AnyRoot,
) -> Result<usize, casual_slide_export::slide::SlideExportError> {
    let xml = OoxmlLimits::default();
    let import = import();
    match to {
        AnyRoot::Dgm(r) => patch_diagram_part(tree, part, false, rels, r, &limits(), &import, &xml),
        AnyRoot::Math(r) => patch_math_part(tree, part, false, rels, r, &limits(), &import, &xml),
        AnyRoot::Ink(r) => patch_ink_part(tree, part, false, rels, r, &limits(), &import, &xml),
    }
}

/// The parts of a deck in each module, standalone or not.
fn parts_of(pkg: &PresentationPackage) -> Vec<(Kind, String)> {
    pkg.part_names()
        .into_iter()
        .filter(|n| n.ends_with(".xml"))
        .filter_map(|n| {
            if n.contains("/diagrams/") {
                Some((Kind::Diagram, n))
            } else if n.contains("/math/") {
                Some((Kind::Math, n))
            } else if n.contains("/ink/") {
                Some((Kind::Ink, n))
            } else {
                None
            }
        })
        .collect()
}

const EVERY_ROW: [&str; 3] = [
    "generated/graphics-diagram-every-row.pptx",
    "generated/graphics-math.pptx",
    "generated/graphics-ink.pptx",
];
const WITH_DIAGRAMS: [&str; 4] = [
    "generated/graphics-diagram-every-row.pptx",
    "generated/graphics-diagram-consistent.pptx",
    "generated/rr-rich-clone.pptx",
    "real/z20089741-tasamoh-ar.pptx",
];

/// `WR-01`: every part, emptied and written from its model alone, reads as what it was;
/// and for the decks built to hold everything, the whole tree comes back.
#[test]
fn every_part_is_a_fixed_point_through_a_forced_rewrite() {
    let mut written = 0;
    for deck in EVERY_ROW.iter().chain(&WITH_DIAGRAMS) {
        let mut pkg = open(fixture(deck));
        for (kind, name) in parts_of(&pkg) {
            let tree = part_tree(&mut pkg, &name);
            let rels = pkg.relationships_of(&name).to_vec();
            let Some(model) = read_any(kind, &tree, &name, &rels) else {
                continue;
            };
            let mut blank = emptied(&tree);
            // The real deck's diagrams hold nothing the blank part refuses.
            patch_any(&mut blank, &name, &rels, &model)
                .unwrap_or_else(|e| panic!("{deck} {name}: {e}"));
            let again = XmlTree::parse(&blank.to_bytes(), &name, &OoxmlLimits::default()).unwrap();
            assert_eq!(
                read_any(kind, &again, &name, &rels).as_ref(),
                Some(&model),
                "{deck} {name}"
            );
            // A second pass changes nothing.
            let mut third = again.clone();
            assert_eq!(patch_any(&mut third, &name, &rels, &model).unwrap(), 0);
            assert_eq!(third.to_bytes(), again.to_bytes());
            written += 1;
        }
    }
    assert!(written > 60, "{written}");
}

/// What the walks read of the checklist: the element and attribute rows of an area.
fn rows(area: &str) -> Vec<(String, String, String, String)> {
    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            (c.get(1) == Some(&area) && matches!(c.get(3), Some(&"element") | Some(&"attribute")))
                .then(|| {
                    (
                        c[0].to_owned(),
                        c[2].to_owned(),
                        c[3].to_owned(),
                        c[4].to_owned(),
                    )
                })
        })
        .collect()
}

/// Every element and attribute name each part of the every-row decks holds, once: take it
/// out and the part must read otherwise, write the original back and the tree must return.
fn walk_names(deck: &str, kind: Kind, seen: &mut BTreeSet<String>, silent: &mut Vec<String>) {
    let mut pkg = open(fixture(deck));
    let mut tried = BTreeSet::new();
    for (k, name) in parts_of(&pkg) {
        if !matches!(
            (k, kind),
            (Kind::Diagram, Kind::Diagram) | (Kind::Math, Kind::Math) | (Kind::Ink, Kind::Ink)
        ) {
            continue;
        }
        let tree = part_tree(&mut pkg, &name);
        let rels = pkg.relationships_of(&name).to_vec();
        let Some(model) = read_any(kind, &tree, &name, &rels) else {
            continue;
        };
        let mut stack = vec![(tree.root(), None)];
        while let Some((id, parent)) = stack.pop() {
            let qname = tree.qualified_name(id).to_owned();
            let local = qname.rsplit(':').next().unwrap_or(&qname).to_owned();
            seen.insert(local.clone());
            for (attr, _) in tree.declared_attributes(id) {
                if attr == "xmlns" || attr.starts_with("xmlns:") {
                    continue;
                }
                let bare = attr.rsplit(':').next().unwrap_or(&attr).to_owned();
                seen.insert(format!("@{bare}"));
                if !tried.insert(format!("{qname}/@{attr}")) {
                    continue;
                }
                let mut without = tree.clone();
                without.remove_attribute(id, &attr);
                let gone = XmlTree::parse(&without.to_bytes(), &name, &OoxmlLimits::default())
                    .unwrap();
                let read = read_any(kind, &gone, &name, &rels);
                if read.as_ref() == Some(&model) {
                    silent.push(format!("{name}: {qname}/@{attr}"));
                } else {
                    // Written back from the model: the attribute's writer, and only it.
                    let mut back = gone.clone();
                    patch_any(&mut back, &name, &rels, &model)
                        .unwrap_or_else(|e| panic!("{name}: {qname}/@{attr}: {e}"));
                    assert_eq!(canon(&back), canon(&tree), "{name}: {qname}/@{attr}");
                }
            }
            if let Some(parent) = parent
                && tried.insert(format!("{qname} in {}", tree.qualified_name(parent)))
                && qname != "a:ext"
                && !qname.starts_with("a:")
            {
                let mut without = tree.clone();
                without.remove_child(parent, id);
                let gone = XmlTree::parse(&without.to_bytes(), &name, &OoxmlLimits::default())
                    .unwrap();
                let read = read_any(kind, &gone, &name, &rels);
                if read.as_ref() == Some(&model) {
                    silent.push(format!("{name}: {qname}"));
                } else {
                    let mut back = gone.clone();
                    patch_any(&mut back, &name, &rels, &model)
                        .unwrap_or_else(|e| panic!("{name}: {qname}: {e}"));
                    // Each element came back with what it held.
                    let again = XmlTree::parse(&back.to_bytes(), &name, &OoxmlLimits::default())
                        .unwrap();
                    assert_eq!(
                        read_any(kind, &again, &name, &rels).as_ref(),
                        Some(&model),
                        "{name}: {qname}"
                    );
                }
            }
            for kid in tree.child_elements(id) {
                stack.push((kid, Some(id)));
            }
        }
    }
}

/// The silent-drop test of `docs/06` gate 6, and each row's writer, in one walk. A row of
/// the area that no fixture can hold is named, with why.
#[test]
fn every_row_is_read_and_written_and_none_is_dropped_silently() {
    /// The rows no presentation part can hold: Word's, in OMML.
    const NOT_HELD: [&str; 3] = [
        "m:CT_OMathArg/EG_PContentMath",
        "m:CT_R/EG_RPr",
        "m:CT_CtrlPr/EG_RPrMath",
    ];
    for (area, deck, kind) in [
        (
            "DGM",
            "generated/graphics-diagram-every-row.pptx",
            Kind::Diagram,
        ),
        ("MATH", "generated/graphics-math.pptx", Kind::Math),
        ("INK", "generated/graphics-ink.pptx", Kind::Ink),
    ] {
        let mut seen = BTreeSet::new();
        let mut silent = Vec::new();
        walk_names(deck, kind, &mut seen, &mut silent);
        assert!(silent.is_empty(), "{area}: read past: {silent:#?}");
        let mut missing = Vec::new();
        for (id, _owner, row_kind, name) in rows(area) {
            if NOT_HELD.contains(&id.as_str()) {
                continue;
            }
            let bare = name.trim_start_matches('@');
            let bare = bare.rsplit(':').next().unwrap_or(bare);
            let wanted = if row_kind == "attribute" {
                format!("@{bare}")
            } else {
                bare.to_owned()
            };
            if !seen.contains(&wanted) {
                missing.push(id);
            }
        }
        assert!(missing.is_empty(), "{area}: {missing:#?}");
    }
}

/// An element the reader does not know, anywhere, is reported and kept.
#[test]
fn what_nothing_models_is_reported_and_kept() {
    let (mut pkg, _, _, d) = first_diagram(&fixture("generated/graphics-diagram-consistent.pptx"));
    let name = d.part(DiagramRole::Data).unwrap().to_owned();
    let bytes = pkg.read_part(&name).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let xml = text.replacen(
        "<dgm:ptLst>",
        "<dgm:unknownKid/><dgm:ptLst odd=\"1\">",
        1,
    );
    let tree = XmlTree::parse(xml.as_bytes(), &name, &OoxmlLimits::default()).unwrap();
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let rels = pkg.relationships_of(&name).to_vec();
    let read = import_diagram_root(&tree, &name, &rels, &limits(), &import, &mut findings)
        .unwrap()
        .unwrap();
    assert!(findings.iter().any(|f| f.kind == K::RetainedElement));
    assert!(findings.has(K::RetainedAttribute, "ptLst"));
    // An edit elsewhere leaves it where it was.
    let dgm::Root::DataModel(mut data) = read else {
        panic!()
    };
    data.pt_lst.as_mut().unwrap().pt[1].type_ = Some(dgm::PtType::Asst);
    let mut edited = tree.clone();
    patch_diagram_part(
        &mut edited,
        &name,
        false,
        &rels,
        &dgm::Root::DataModel(data),
        &limits(),
        &import,
        &OoxmlLimits::default(),
    )
    .unwrap();
    let out = String::from_utf8(edited.to_bytes()).unwrap();
    assert!(out.contains("<dgm:unknownKid/>"));
    assert!(out.contains("odd=\"1\""));
}

#[test]
fn an_unedited_save_returns_the_same_bytes() {
    for deck in WITH_DIAGRAMS {
        let source = fixture(deck);
        let mut pkg = open(source.clone());
        for (part, ids) in frames(&mut pkg) {
            let (d, _) = diagram_of(&mut pkg, &part, &ids);
            let saved = save_diagram(&mut pkg, &part, &ids, &d, &limits(), &import()).unwrap();
            assert_eq!(saved.outcome.bytes, source, "{deck}");
            assert!(saved.outcome.changed_parts.is_empty());
            assert!(!saved.stale_drawing);
        }
    }
}

/// `T1K-01`'s acceptance: the five parts stay consistent.
#[test]
fn the_five_parts_agree_and_a_data_edit_keeps_them_so() {
    let source = fixture("generated/graphics-diagram-consistent.pptx");
    let (mut pkg, part, ids, d) = first_diagram(&source);
    assert_eq!(d.inconsistencies(), BTreeSet::new());
    for role in DiagramRole::ALL {
        assert!(d.part(role).is_some(), "{role:?}");
    }
    assert!(d.set_aside.is_empty());
    let point = "{00000000-0000-4000-8000-000000000001}";
    let mut after = d.clone();
    assert!(after.set_point_text(point, "Second"));
    assert_eq!(after.inconsistencies(), BTreeSet::new());
    let saved = save_diagram(&mut pkg, &part, &ids, &after, &limits(), &import()).unwrap();
    assert!(!saved.stale_drawing);
    let data = d.part(DiagramRole::Data).unwrap().to_owned();
    let drawing = d.part(DiagramRole::Drawing).unwrap().to_owned();
    assert_eq!(saved.outcome.changed_parts, vec![data.clone(), drawing.clone()]);
    // Only the two text nodes, byte for byte, in two parts of five.
    let mut before_pkg = open(source);
    let mut out = open(saved.outcome.bytes.clone());
    for name in [&data, &drawing] {
        let old = String::from_utf8(before_pkg.read_part(name).unwrap()).unwrap();
        let new = String::from_utf8(out.read_part(name).unwrap()).unwrap();
        assert_eq!(old.replace("First", "Second"), new, "{name}");
    }
    let (read, _) = diagram_of(&mut out, &part, &ids);
    assert_eq!(read.data, after.data);
    assert_eq!(read.drawing, after.drawing);
    assert_eq!(read.layout, d.layout);
    f4_validates(&saved.outcome.bytes);
    let dir = edited_dir();
    if let Some(dir) = dir {
        std::fs::write(dir.join("diagram-text.pptx"), &saved.outcome.bytes).unwrap();
    }
}

/// `INV-FB-01`.
#[test]
fn a_data_edit_the_drawing_does_not_show_is_blocked_and_nothing_is_saved() {
    let (mut pkg, part, ids, d) =
        first_diagram(&fixture("generated/graphics-diagram-consistent.pptx"));
    let mut text_only = d.clone();
    let pt = &mut text_only.data.as_mut().unwrap().pt_lst.as_mut().unwrap().pt[1];
    let body = pt.t.as_mut().unwrap();
    let TextItem::Run(run) = &mut body.paragraphs[0].items[0] else {
        panic!()
    };
    run.text = Some("Second".to_owned());
    let err = save_diagram(&mut pkg, &part, &ids, &text_only, &limits(), &import()).unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0009", "{err}");

    let mut structural = d.clone();
    structural
        .data
        .as_mut()
        .unwrap()
        .pt_lst
        .as_mut()
        .unwrap()
        .pt
        .push(dgm::Pt {
            model_id: Some("99".to_owned()),
            ..dgm::Pt::default()
        });
    let err = save_diagram(&mut pkg, &part, &ids, &structural, &limits(), &import()).unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0009", "{err}");
}

/// `CS-SAV-0007`: kept, and said so.
#[test]
fn a_definition_edit_keeps_the_drawing_and_says_it_is_stale() {
    let (mut pkg, part, ids, d) =
        first_diagram(&fixture("generated/graphics-diagram-consistent.pptx"));
    let mut after = d.clone();
    after.colors.as_mut().unwrap().style_lbl[0]
        .fill_clr_lst
        .as_mut()
        .unwrap()
        .meth = Some(dgm::ClrAppMethod::Span);
    let saved = save_diagram(&mut pkg, &part, &ids, &after, &limits(), &import()).unwrap();
    assert!(saved.stale_drawing);
    assert_eq!(
        saved.outcome.changed_parts,
        vec![d.part(DiagramRole::Colors).unwrap().to_owned()]
    );
    f4_validates(&saved.outcome.bytes);
    if let Some(dir) = edited_dir() {
        std::fs::write(dir.join("diagram-colors.pptx"), &saved.outcome.bytes).unwrap();
    }
}

/// A part that is not XML: set aside and reported, the others read, the deck opens, and
/// it comes back byte for byte. An edit of it is refused.
#[test]
fn a_damaged_part_is_set_aside_and_the_rest_reads() {
    let source = fixture("generated/graphics-diagram-consistent.pptx");
    let (mut pkg, part, ids, d) = first_diagram(&source);
    let layout = d.part(DiagramRole::Layout).unwrap().to_owned();
    let mut edits = PartEdits::default();
    edits.replace(layout.clone(), b"<not xml".to_vec());
    let damaged = pkg.save_with(&edits).unwrap();

    let mut pkg = open(damaged.clone());
    let (read, findings) = diagram_of(&mut pkg, &part, &ids);
    assert!(read.is_set_aside(DiagramRole::Layout));
    assert!(read.layout.is_none());
    assert_eq!(read.data, d.data);
    assert_eq!(read.drawing, d.drawing);
    assert_eq!(read.style, d.style);
    assert_eq!(read.colors, d.colors);
    assert!(findings.iter().any(|f| f.kind.code() == "CS-GFF-0001"));
    assert!(read.inconsistencies().contains(&Inconsistency::MissingPart(DiagramRole::Layout)));

    // The deck's slide still reads, as nothing it holds was read with it.
    let s = slide(&mut pkg, SLIDE);
    assert!(s.shape_tree().unwrap().children[0].diagram.is_some());
    // Unedited: the same bytes.
    let same = save_diagram(&mut pkg, &part, &ids, &read, &limits(), &import()).unwrap();
    assert_eq!(same.outcome.bytes, damaged);
    // An edit elsewhere keeps it as it was.
    let mut after = read.clone();
    assert!(after.set_point_text("{00000000-0000-4000-8000-000000000001}", "Second"));
    let saved = save_diagram(&mut pkg, &part, &ids, &after, &limits(), &import()).unwrap();
    let mut out = open(saved.outcome.bytes);
    assert_eq!(out.read_part(&layout).unwrap(), b"<not xml");
    // An edit of it is refused: the model has nothing to write into.
    let mut asked = read;
    asked.layout = d.layout;
    let err = save_diagram(&mut pkg, &part, &ids, &asked, &limits(), &import()).unwrap_err();
    assert_eq!(err.code(), "CS-SLE-0001", "{err}");
}

/// Every hostile deck opens, with what crossed a bound set aside under its code.
#[test]
fn a_part_over_a_bound_is_set_aside_with_its_code() {
    for (deck, role, code) in [
        ("hostile/graphics-limit-points.bin", DiagramRole::Data, "CS-GFX-0003"),
        ("hostile/graphics-limit-connections.bin", DiagramRole::Data, "CS-GFX-0004"),
        ("hostile/graphics-limit-layout-nodes.bin", DiagramRole::Layout, "CS-GFX-0005"),
        ("hostile/graphics-limit-drawing-shapes.bin", DiagramRole::Drawing, "CS-GFX-0006"),
        ("hostile/graphics-limit-depth.bin", DiagramRole::Layout, "CS-GFX-0001"),
        ("hostile/graphics-wrong-root.bin", DiagramRole::Style, "CS-IMP-0003"),
        ("hostile/graphics-wrong-root.bin", DiagramRole::Colors, "CS-IMP-0003"),
    ] {
        let mut pkg = open(fixture(deck));
        let (part, ids) = frames(&mut pkg).remove(0);
        let (d, findings) = diagram_of(&mut pkg, &part, &ids);
        let set = d
            .set_aside
            .iter()
            .find(|(r, _)| *r == role)
            .unwrap_or_else(|| panic!("{deck}: {:?}", d.set_aside));
        assert_eq!(set.1.code, code, "{deck}");
        assert!(findings.iter().any(|f| f.kind.code() == "CS-GFF-0001"));
    }
}

/// A bound too large for a fixture, lowered, and a bound met exactly.
#[test]
fn the_element_bound_and_each_bound_at_its_value() {
    let (mut pkg, part, ids, d) =
        first_diagram(&fixture("generated/graphics-diagram-consistent.pptx"));
    let import = import();
    let read = |pkg: &mut PresentationPackage, limits: GraphicLimits| {
        let mut findings = Findings::new(import.max_findings);
        import_diagram(pkg, &part, &ids, &limits, &import, &mut findings)
    };
    let few = GraphicLimits {
        max_elements: 3,
        ..GraphicLimits::default()
    };
    let got = read(&mut pkg, few);
    assert_eq!(got.set_aside[0].1.code, "CS-GFX-0002");
    // Its five points, exactly: over by none.
    let five = GraphicLimits {
        max_points: 5,
        ..GraphicLimits::default()
    };
    assert_eq!(read(&mut pkg, five).data, d.data);
    let four = GraphicLimits {
        max_points: 4,
        ..GraphicLimits::default()
    };
    assert_eq!(read(&mut pkg, four).set_aside[0].1.code, "CS-GFX-0003");
}

/// `SEC-C12`. A definition is data: a loop of the largest count, a hundred deep, reads as its
/// size. Were anything to run it, this would not finish.
#[test]
fn a_layout_definition_is_read_and_never_run() {
    let mut pkg = open(fixture("hostile/graphics-layout-loop-bomb.bin"));
    let (part, ids) = frames(&mut pkg).remove(0);
    let (d, _) = diagram_of(&mut pkg, &part, &ids);
    let layout = d.layout.as_ref().expect("read, as data");
    let dgm::LayoutNodeItem::ForEach(each) = &layout.layout_node.as_ref().unwrap().items[0] else {
        panic!()
    };
    assert_eq!(each.cnt.as_ref().unwrap().0, vec![4_294_967_295]);
    // It writes back as what it is, too.
    let saved = save_diagram(&mut pkg, &part, &ids, &d, &limits(), &import()).unwrap();
    assert!(saved.outcome.changed_parts.is_empty());
}

/// Damage across the parts: reported, kept, and no document text in the report.
#[test]
fn what_the_parts_disagree_on_is_reported_and_nothing_is_repaired() {
    let mut pkg = open(fixture("hostile/graphics-bad-data.bin"));
    let (part, ids) = frames(&mut pkg).remove(0);
    let (d, findings) = diagram_of(&mut pkg, &part, &ids);
    let found = d.inconsistencies();
    assert!(found.contains(&Inconsistency::DuplicatePoint));
    assert!(found.contains(&Inconsistency::DanglingConnection));
    // The drawing's relationship is dangling, so it is not read at all.
    assert!(d.drawing.is_none());
    let codes: BTreeSet<&str> = findings.iter().map(|f| f.kind.code()).collect();
    for code in ["CS-GFF-0003", "CS-GFF-0004", "CS-IMF-0006"] {
        assert!(codes.contains(code), "{code}: {codes:?}");
    }
    for f in findings.iter() {
        assert!(!format!("{f:?}").contains("QzMarkerQz"), "{f:?}");
    }
}

/// A diagram the real deck holds: all five parts, in agreement, with its drawing found by
/// `dsp:dataModelExt`.
#[test]
fn real_smartart_reads_as_five_parts() {
    let mut pkg = open(fixture("real/z20089741-tasamoh-ar.pptx"));
    let list = frames(&mut pkg);
    assert!(!list.is_empty());
    let mut points = 0;
    let mut shapes = 0;
    for (part, ids) in &list {
        let (d, _) = diagram_of(&mut pkg, part, ids);
        assert!(d.set_aside.is_empty(), "{:?}", d.set_aside);
        for role in DiagramRole::ALL {
            assert!(d.part(role).is_some(), "{part}: {role:?}");
        }
        assert_eq!(
            d.inconsistencies()
                .into_iter()
                .filter(|i| matches!(i, Inconsistency::MissingPart(_)))
                .count(),
            0
        );
        points += d.points().len();
        shapes += d.drawing_shapes().len();
    }
    assert!(points > 10, "{points}");
    assert!(shapes > 5, "{shapes}");
}

fn inline_of(part: &SlidePart, name: &str) -> Vec<TextItem> {
    let node = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| {
            n.non_visual
                .as_ref()
                .is_some_and(|v| v.name.as_deref() == Some(name))
        })
        .unwrap();
    node.text_body.as_ref().unwrap().paragraphs[0].items.clone()
}

/// `T1F-04`: an equation in `mc:AlternateContent` is no longer a name. Both branches are
/// in the model, with Office's `a:rPr` runs.
#[test]
fn an_inline_equation_holds_both_its_branches() {
    let mut pkg = open(fixture("generated/graphics-math.pptx"));
    let part = slide(&mut pkg, SLIDE);
    let items = inline_of(&part, "Inline");
    assert!(matches!(items[0], TextItem::Run(_)));
    assert!(matches!(items[2], TextItem::Run(_)));
    let TextItem::Math(math) = &items[1] else {
        panic!("{:?}", items[1])
    };
    let alternate = math.alternate.as_ref().unwrap();
    assert_eq!(alternate.requires.as_deref(), Some("a14"));
    let fallback = alternate.fallback.as_ref().unwrap();
    let TextItem::Run(run) = &fallback[0] else {
        panic!()
    };
    assert_eq!(run.text.as_deref(), Some("a/b + c"));
    let MathItem::OMathPara(para) = &math.math.as_ref().unwrap().items[0] else {
        panic!()
    };
    let equation = &para.o_math[0];
    let omml::OMathItem::F(fraction) = &equation.items[0] else {
        panic!()
    };
    let omml::OMathArgItem::R(run) = &fraction.num.as_ref().unwrap().items[0] else {
        panic!()
    };
    assert!(run.a_r_pr.is_some());
    let omml::RItem::T(t) = &run.items[0];
    assert_eq!(t.text.as_deref(), Some("a"));
    // A control's properties are held too.
    let ctrl = fraction.f_pr.as_ref().unwrap().ctrl_pr.as_ref().unwrap();
    assert!(ctrl.a_r_pr.is_some());
}

/// `T1F-04`'s refused bodies: emptied, written from the model, and the same tree.
#[test]
fn a_body_holding_an_equation_is_written_from_the_model() {
    let source = fixture("generated/graphics-math.pptx");
    let mut pkg = open(source);
    let bytes = pkg.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let rels = pkg.relationships_of(SLIDE).to_vec();
    let part = read_slide(&tree, SLIDE, &rels);
    let mut empty = part.clone();
    let ids: Vec<_> = part
        .shape_tree()
        .unwrap()
        .descendants()
        .filter(|n| n.text_body.is_some() && n.non_visual.as_ref().unwrap().name.as_deref() != Some("Whole"))
        .map(|n| n.id)
        .collect();
    for id in &ids {
        empty.node_mut(*id).unwrap().text_body = None;
    }
    let (xml, slide_limits, import) = (OoxmlLimits::default(), SlideLimits::default(), import());
    let mut written = tree.clone();
    let patched = |t: &mut XmlTree, to: &SlidePart| {
        patch_slide_part(t, SLIDE, false, &rels, to, &slide_limits, &import, &xml)
    };
    patched(&mut written, &empty).unwrap();
    patched(&mut written, &part).unwrap();
    let again = XmlTree::parse(&written.to_bytes(), SLIDE, &xml).unwrap();
    assert_eq!(read_slide(&again, SLIDE, &rels), part);
}

/// `INV-FB-03`, and the edits it allows.
#[test]
fn an_equation_edit_is_held_to_its_fallback() {
    let source = fixture("generated/graphics-math.pptx");
    let mut pkg = open(source);
    let bytes = pkg.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let rels = pkg.relationships_of(SLIDE).to_vec();
    let part = read_slide(&tree, SLIDE, &rels);
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().unwrap().name.as_deref() == Some("Inline"))
        .unwrap()
        .id;
    let edit = |math_too: bool, fallback_too: bool| {
        let mut after = part.clone();
        let items = &mut after.node_mut(id).unwrap().text_body.as_mut().unwrap().paragraphs[0].items;
        let TextItem::Math(math) = &mut items[1] else {
            panic!()
        };
        if math_too {
            let MathItem::OMathPara(para) = &mut math.math.as_mut().unwrap().items[0] else {
                panic!()
            };
            let eq = &mut para.o_math[0];
            let omml::OMathItem::R(run) = &mut eq.items[1] else {
                panic!()
            };
            let omml::RItem::T(t) = &mut run.items[0];
            t.text = Some(" + d".to_owned());
        }
        if fallback_too {
            let fallback = math.alternate.as_mut().unwrap().fallback.as_mut().unwrap();
            let TextItem::Run(run) = &mut fallback[0] else {
                panic!()
            };
            run.text = Some("a/b + d".to_owned());
        }
        let mut t = tree.clone();
        patch_slide_part(
            &mut t,
            SLIDE,
            false,
            &rels,
            &after,
            &SlideLimits::default(),
            &import(),
            &OoxmlLimits::default(),
        )
        .map(|_| String::from_utf8(t.to_bytes()).unwrap())
        .map_err(|e| e.code())
    };
    assert_eq!(edit(true, false).unwrap_err(), "CS-EXP-0009");
    let old = String::from_utf8(bytes.clone()).unwrap();
    let both = edit(true, true).unwrap();
    assert_eq!(
        old.replace("> + c</m:t>", "> + d</m:t>").replace("a/b + c", "a/b + d"),
        both
    );
    // Only the fallback: the equation is what is true, so that is allowed.
    edit(false, true).unwrap();
    // An edit is valid as a whole deck.
    let mut edits = PartEdits::default();
    edits.replace(SLIDE, both.into_bytes());
    let deck = pkg.save_with(&edits).unwrap();
    f4_validates(&deck);
    if let Some(dir) = edited_dir() {
        std::fs::write(dir.join("math-inline.pptx"), &deck).unwrap();
    }
}

/// PowerPoint's own form: the whole shape in the `mc:Choice`. The fallback shape is the node;
/// the equation is read beside it, and an edit of it is blocked.
#[test]
fn an_equation_in_a_branch_not_taken_is_read_and_its_edit_blocked() {
    let mut pkg = open(fixture("generated/graphics-math.pptx"));
    let bytes = pkg.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let rels = pkg.relationships_of(SLIDE).to_vec();
    let part = read_slide(&tree, SLIDE, &rels);
    let root = part.shape_tree().unwrap();
    assert_eq!(root.alternates.len(), 1);
    let branch = &root.alternates[0];
    assert_eq!(branch.requires.as_deref(), Some("a14"));
    assert_eq!(branch.equations.len(), 1);
    let mut after = part.clone();
    let root_id = root.id;
    after.node_mut(root_id).unwrap().alternates[0].equations.clear();
    let mut t = tree.clone();
    let err = patch_slide_part(
        &mut t,
        SLIDE,
        false,
        &rels,
        &after,
        &SlideLimits::default(),
        &import(),
        &OoxmlLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-SAV-0009", "{err}");
}

/// A bare `a14:m` in a paragraph, which a file marking `a14` ignorable may hold.
#[test]
fn a_bare_equation_reads_and_writes() {
    const NS: &str = concat!(
        r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
        r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
        r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" "#,
        r#"xmlns:a14="http://schemas.microsoft.com/office/drawing/2010/main" "#,
        r#"xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math""#
    );
    let xml = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="B"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:p><a14:m><m:oMath><m:r><m:t>x</m:t></m:r></m:oMath></a14:m></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
    );
    let tree = XmlTree::parse(xml.as_bytes(), SLIDE, &OoxmlLimits::default()).unwrap();
    let part = read_slide(&tree, SLIDE, &[]);
    let items = inline_of(&part, "B");
    let TextItem::Math(math) = &items[0] else {
        panic!("{items:?}")
    };
    assert!(math.alternate.is_none());
    let mut after = part.clone();
    let id = part.shape_tree().unwrap().children[0].id;
    after.node_mut(id).unwrap().text_body.as_mut().unwrap().paragraphs[0].items = vec![];
    let mut t = tree.clone();
    let patched = |t: &mut XmlTree, to: &SlidePart| {
        patch_slide_part(
            t,
            SLIDE,
            false,
            &[],
            to,
            &SlideLimits::default(),
            &import(),
            &OoxmlLimits::default(),
        )
    };
    patched(&mut t, &after).unwrap();
    assert!(!String::from_utf8(t.to_bytes()).unwrap().contains("a14:m"));
    patched(&mut t, &part).unwrap();
    assert!(String::from_utf8(t.to_bytes()).unwrap().contains("a14:m"));
}

/// An equation over the bound stays in the tree, and the slide reads.
#[test]
fn an_equation_over_a_bound_stays_in_the_tree_and_is_reported() {
    let mut pkg = open(fixture("hostile/graphics-limit-math-depth.bin"));
    let bytes = pkg.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        SLIDE,
        &[],
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .unwrap();
    let items = inline_of(&part, "Deep");
    assert!(matches!(&items[0], TextItem::Retained(n) if n == "AlternateContent"));
    assert!(findings.iter().any(|f| f.kind.code() == "CS-GFF-0008"));
}

/// Ink through its content part: the strokes read, and the fallback picture is the node.
#[test]
fn ink_reads_through_its_content_part_with_the_fallback_kept() {
    let source = fixture("generated/graphics-ink.pptx");
    let mut pkg = open(source.clone());
    let part = slide(&mut pkg, SLIDE);
    let root = part.shape_tree().unwrap();
    // The fallback is the shape, as markup compatibility has it.
    assert_eq!(root.children[0].kind, NodeKind::Picture);
    assert_eq!(root.alternates[0].content_parts, vec!["rId2".to_owned()]);
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let list = import_slide_ink(&mut pkg, SLIDE, &part, &limits(), &import, &mut findings);
    assert_eq!(list.len(), 1);
    let read = &list[0].1;
    assert!(read.set_aside.is_none());
    let Some(ink::Root::Ink(doc)) = &read.root else {
        panic!("{:?}", read.root)
    };
    let stroke = doc.items.iter().find_map(|i| match i {
        ink::InkItem::Trace(t) => Some(t),
        _ => None,
    });
    assert_eq!(stroke.unwrap().text.as_deref(), Some("1 2 3, 4 5 6"));
    // Unedited: the same bytes.
    let same = save_ink_part(&mut pkg, read, &limits(), &import).unwrap();
    assert_eq!(same.outcome.bytes, source);
    assert!(!same.stale_fallback);

    // A stroke edit: one part, one text node, and the host told what went stale.
    let mut after = read.clone();
    let Some(ink::Root::Ink(doc)) = &mut after.root else {
        panic!()
    };
    for item in &mut doc.items {
        if let ink::InkItem::Trace(t) = item {
            t.text = Some("7 8 9".to_owned());
        }
    }
    let saved = save_ink_part(&mut pkg, &after, &limits(), &import).unwrap();
    assert!(saved.stale_fallback);
    assert_eq!(saved.outcome.changed_parts, vec![read.part.clone()]);
    let mut out = open(saved.outcome.bytes.clone());
    let old = String::from_utf8(pkg.read_part(&read.part).unwrap()).unwrap();
    let new = String::from_utf8(out.read_part(&read.part).unwrap()).unwrap();
    assert_eq!(old.replace("1 2 3, 4 5 6", "7 8 9"), new);
    assert_eq!(out.read_part(SLIDE).unwrap(), pkg.read_part(SLIDE).unwrap());
    f4_validates(&saved.outcome.bytes);
    if let Some(dir) = edited_dir() {
        std::fs::write(dir.join("ink-trace.pptx"), &saved.outcome.bytes).unwrap();
    }
}

#[test]
fn a_damaged_ink_part_is_set_aside_and_the_slide_stands() {
    let source = fixture("generated/graphics-ink.pptx");
    let mut pkg = open(source);
    let mut edits = PartEdits::default();
    edits.replace("ppt/ink/ink0.xml", b"<nope".to_vec());
    let damaged = pkg.save_with(&edits).unwrap();
    let mut pkg = open(damaged.clone());
    let import = import();
    let mut findings = Findings::new(import.max_findings);
    let read = import_ink_part(&mut pkg, "ppt/ink/ink0.xml", &limits(), &import, &mut findings);
    assert!(read.root.is_none());
    assert!(read.set_aside.is_some());
    assert!(findings.iter().any(|f| f.kind.code() == "CS-GFF-0001"));
    assert!(slide(&mut pkg, SLIDE).shape_tree().is_some());
    let same = save_ink_part(&mut pkg, &read, &limits(), &import).unwrap();
    assert_eq!(same.outcome.bytes, damaged);
    // Over a bound: set aside under its code.
    let mut pkg = open(fixture("hostile/graphics-limit-traces.bin"));
    let read = import_ink_part(&mut pkg, "ppt/ink/ink1.xml", &limits(), &import, &mut findings);
    assert_eq!(read.set_aside.unwrap().code, "CS-GFX-0007");
}

/// A frame's diagram references are patched in place; the frame is in the slide.
#[test]
fn a_frames_diagram_references_are_read_and_patched() {
    let mut pkg = open(fixture("generated/graphics-diagram-consistent.pptx"));
    let bytes = pkg.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&bytes, SLIDE, &OoxmlLimits::default()).unwrap();
    let rels = pkg.relationships_of(SLIDE).to_vec();
    let part = read_slide(&tree, SLIDE, &rels);
    let frame = &part.shape_tree().unwrap().children[0];
    let ids = frame.diagram.as_ref().unwrap();
    assert_eq!(ids.dm.as_deref(), Some("rId2"));
    let import_limits = import();
    let mut findings = Findings::new(import_limits.max_findings);
    let all = import_slide_diagrams(&mut pkg, SLIDE, &part, &limits(), &import_limits, &mut findings);
    assert_eq!(all.len(), 1);

    let mut after = part.clone();
    after.node_mut(frame.id).unwrap().diagram.as_mut().unwrap().lo = Some("rId6".to_owned());
    let mut t = tree.clone();
    let (xml, slide_limits) = (OoxmlLimits::default(), SlideLimits::default());
    let patched = |t: &mut XmlTree, to: &SlidePart| {
        patch_slide_part(t, SLIDE, false, &rels, to, &slide_limits, &import_limits, &xml)
    };
    patched(&mut t, &after).unwrap();
    let old = String::from_utf8(bytes).unwrap();
    assert_eq!(
        old.replace("r:lo=\"rId3\"", "r:lo=\"rId6\""),
        String::from_utf8(t.to_bytes()).unwrap()
    );
    // A relationship the part does not have is refused.
    after.node_mut(frame.id).unwrap().diagram.as_mut().unwrap().lo = Some("rId99".to_owned());
    assert_eq!(patched(&mut tree.clone(), &after).unwrap_err().code(), "CS-EXP-0003");
    // And adding or removing one is an operation this tier does not have.
    after.node_mut(frame.id).unwrap().diagram = None;
    assert_eq!(patched(&mut tree.clone(), &after).unwrap_err().code(), "CS-SLE-0001");
}

/// Strict namespaces read and write the same.
#[test]
fn strict_namespaces_read_as_transitional_and_new_nodes_use_them() {
    let mut pkg = open(fixture("generated/graphics-diagram-consistent.pptx"));
    let name = "ppt/diagrams/colors1.xml";
    let text = String::from_utf8(pkg.read_part(name).unwrap()).unwrap();
    let strict = text
        .replace(
            "http://schemas.openxmlformats.org/drawingml/2006/diagram",
            "http://purl.oclc.org/ooxml/drawingml/diagram",
        )
        .replace(
            "http://schemas.openxmlformats.org/drawingml/2006/main",
            "http://purl.oclc.org/ooxml/drawingml/main",
        );
    let tree = XmlTree::parse(strict.as_bytes(), name, &OoxmlLimits::default()).unwrap();
    let plain = XmlTree::parse(text.as_bytes(), name, &OoxmlLimits::default()).unwrap();
    assert_eq!(
        read_dgm_root(&tree, name, &[]).unwrap(),
        read_dgm_root(&plain, name, &[]).unwrap()
    );
}

/// The edited decks' validation, by `tools/ooxml-sdk/validate.py`, runs on what this writes.
fn edited_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("OPENPRESENT_EDITED_DIR")?;
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    Some(dir)
}

fn f4_validates(bytes: &[u8]) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-graphics");
    std::fs::create_dir_all(&dir).unwrap();
    let hash: u64 = bytes.iter().fold(7u64, |h, b| {
        h.wrapping_mul(31).wrapping_add(u64::from(*b))
    });
    let file = dir.join(format!("edited-{hash}.pptx"));
    std::fs::write(&file, bytes).unwrap();
    let tool = repo().join("tools/f4/validate.py");
    let out = Command::new("python3").arg("-I").arg(tool).arg(&file).output();
    let Ok(out) = out else {
        assert!(std::env::var_os("F4_REQUIRED").is_none(), "python3 is missing");
        return;
    };
    std::fs::remove_file(&file).unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
}

/// `docs/20` carries each code a part can be set aside under, by name: the walks above see
/// what the deck's parts do, and this keeps the two lists in one place.
#[test]
fn every_bound_has_its_code() {
    let mut by: BTreeMap<&str, &str> = BTreeMap::new();
    use casual_slide_import::diagram::GraphicBound as B;
    for (b, name) in [
        (B::Depth, "CS-GFX-0001"),
        (B::Elements, "CS-GFX-0002"),
        (B::Points, "CS-GFX-0003"),
        (B::Connections, "CS-GFX-0004"),
        (B::LayoutNodes, "CS-GFX-0005"),
        (B::DrawingShapes, "CS-GFX-0006"),
        (B::Traces, "CS-GFX-0007"),
    ] {
        assert_eq!(b.code(), name);
        by.insert(b.code(), name);
    }
    assert_eq!(by.len(), 7);
}
