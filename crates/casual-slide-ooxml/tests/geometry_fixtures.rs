//! `GEO-04`: hostile decks holding custom geometry past each cap, or built to hurt.
//!
//! Nothing reads `a:custGeom` from a part until `T1E-01`, so this test has its own small
//! reader. Bytes come from the fixtures, through the parts of the engine a real import
//! will use, so that the binding inherits a fixture for each limit. It lives here because
//! the geometry crate depends on nothing, tests included.

use std::path::Path;

use casual_slide_geometry::{
    Diagnostic, Extents, RawAxis, RawCommand, RawGeometry, RawGuide, RawHandle, RawPath, RawPoint,
    RawSite,
};
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

fn attr(tree: &XmlTree, el: ElementId, name: &str) -> String {
    tree.plain_attribute(el, name).unwrap_or_default()
}

fn children<'t>(tree: &'t XmlTree, el: ElementId) -> impl Iterator<Item = ElementId> + 't {
    tree.child_elements(el).into_iter()
}

fn point(tree: &XmlTree, el: ElementId) -> RawPoint {
    RawPoint {
        x: attr(tree, el, "x"),
        y: attr(tree, el, "y"),
    }
}

fn pos(tree: &XmlTree, el: ElementId) -> RawPoint {
    children(tree, el)
        .find(|&c| tree.local_name(c) == "pos")
        .map(|c| point(tree, c))
        .unwrap_or_default()
}

fn first_pt(tree: &XmlTree, el: ElementId) -> RawPoint {
    children(tree, el)
        .next()
        .map(|c| point(tree, c))
        .unwrap_or_default()
}

fn find_cust_geom(tree: &XmlTree, el: ElementId) -> Option<ElementId> {
    if tree.local_name(el) == "custGeom" {
        return Some(el);
    }
    children(tree, el).find_map(|c| find_cust_geom(tree, c))
}

fn read_geometry(name: &str) -> RawGeometry {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/hostile")
        .join(format!("geometry-{name}.bin"));
    let bytes = std::fs::read(path).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let xml = pkg.read_part("ppt/slides/slide1.xml").unwrap();
    let tree = XmlTree::parse(&xml, "slide", &OoxmlLimits::default()).unwrap();
    let cust = find_cust_geom(&tree, tree.root()).unwrap();
    let mut raw = RawGeometry::default();
    for list in children(&tree, cust) {
        for item in children(&tree, list) {
            match tree.local_name(list) {
                "gdLst" => raw.gd.push(RawGuide {
                    name: attr(&tree, item, "name"),
                    fmla: attr(&tree, item, "fmla"),
                }),
                "ahLst" => raw.handles.push(RawHandle::Xy {
                    x: RawAxis::default(),
                    y: RawAxis::default(),
                    pos: pos(&tree, item),
                }),
                "cxnLst" => raw.sites.push(RawSite {
                    ang: attr(&tree, item, "ang"),
                    pos: pos(&tree, item),
                }),
                "pathLst" => {
                    let mut path = RawPath::default();
                    for command in children(&tree, item) {
                        path.commands.push(match tree.local_name(command) {
                            "moveTo" => RawCommand::MoveTo(first_pt(&tree, command)),
                            "lnTo" => RawCommand::LineTo(first_pt(&tree, command)),
                            "arcTo" => RawCommand::ArcTo {
                                wr: attr(&tree, command, "wR"),
                                hr: attr(&tree, command, "hR"),
                                st_ang: attr(&tree, command, "stAng"),
                                sw_ang: attr(&tree, command, "swAng"),
                            },
                            other => panic!("{other}"),
                        });
                    }
                    raw.paths.push(path);
                }
                other => panic!("{other}"),
            }
        }
    }
    raw
}

#[test]
fn a_deck_past_each_cap_refuses_with_its_code() {
    for (name, code) in [
        ("too-many-guides", "CS-GEO-0001"),
        ("too-many-paths", "CS-GEO-0002"),
        ("too-many-points", "CS-GEO-0003"),
        ("too-many-handles", "CS-GEO-0004"),
        ("too-many-sites", "CS-GEO-0005"),
    ] {
        let err = read_geometry(name).compile().unwrap_err();
        assert_eq!(err.code(), code, "{name}: {err}");
    }
}

#[test]
fn names_and_formulas_over_their_length_are_repaired_and_reported() {
    let compiled = read_geometry("long-names").compile().unwrap();
    let kept = compiled.diagnostics.kept();
    assert_eq!(kept, [Diagnostic::TooLong, Diagnostic::TooLong]);
    assert_eq!(kept[0].code(), "CS-GEF-0004");
}

#[test]
fn a_cycle_cannot_be_written_down() {
    let compiled = read_geometry("cycle").compile().unwrap();
    assert_eq!(compiled.diagnostics.kept()[0], Diagnostic::UnknownGuide);
    let out = compiled.def.evaluate(Extents::new(914_400, 914_400), &[]);
    assert_eq!(out.guide("ga"), Some(1.0));
    assert_eq!(out.guide("gb"), Some(2.0));
}

#[test]
fn huge_values_saturate_and_stay_finite() {
    let compiled = read_geometry("huge-values").compile().unwrap();
    let out = compiled.def.evaluate(Extents::new(914_400, 914_400), &[]);
    for name in ["a", "b"] {
        assert!(out.guide(name).unwrap().is_finite(), "{name}");
    }
    assert!(out.fingerprint() != 0);
}

/// Every code in `docs/20`, by construction here, so a new variant needs a line.
#[test]
fn each_repair_has_its_stable_code() {
    for (diagnostic, code) in [
        (Diagnostic::UnknownOperator, "CS-GEF-0001"),
        (Diagnostic::WrongOperandCount, "CS-GEF-0002"),
        (Diagnostic::UnknownGuide, "CS-GEF-0003"),
        (Diagnostic::TooLong, "CS-GEF-0004"),
        (Diagnostic::UnknownAdjust, "CS-GEF-0005"),
        (Diagnostic::TooManyAdjusts, "CS-GEF-0006"),
        (Diagnostic::PathWithoutMoveTo, "CS-GEF-0007"),
    ] {
        assert_eq!(diagnostic.code(), code);
    }
}
