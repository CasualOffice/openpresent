//! A silent drop fails here (`docs/06`, per-tier gate 6). For every element and attribute
//! of every shape in the decks that hold the most of Tiers 1E and 1F: take it out, read the
//! part again, and something must differ, in the model or in the report. If neither
//! differs, the importer read past it and said nothing, which is the one thing it may not
//! do. Slide parts only, and only inside the shape tree; other tiers keep their own.

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const PART: &str = "ppt/slides/slide1.xml";
/// The decks walked. Anything with a body-level `extLst` or a foreign child is here too, as
/// retention must be reported as much as anything else.
const DECKS: [&str; 6] = [
    "drawing-every-element",
    "drawing-extension-lists",
    "text-every-property",
    "text-warp-and-3d",
    "text-object-defaults",
    "shape-fills",
];
/// Read on purpose by something other than its text, each with why.
const NAMELESS: [(&str, &str, &str); 1] = [
    // An extension is matched by what it holds: real decks spell one `uri` many ways.
    ("a:ext", "uri", "CS-DRW docs/22"),
];

/// Required by the schema and holding nothing when empty: taking one away loses nothing, so
/// there is nothing to notice. Each is still kept by the tree.
const EMPTY_HOLDERS: [&str; 6] = [
    "p:spPr",
    "p:cNvSpPr",
    "p:cNvGrpSpPr",
    "p:cNvPicPr",
    "p:cNvCxnSpPr",
    "p:grpSpPr",
];

type Read = (SlidePart, Vec<String>, Vec<String>);

fn read(xml: &[u8]) -> Option<Read> {
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

/// Local names the report names as kept and not modelled, or as anything else.
fn named_in_report((_, report, colors): &Read) -> BTreeSet<String> {
    report
        .iter()
        .chain(colors)
        .filter_map(|line| line.split(' ').nth(1))
        .map(|name| name.rsplit(':').next().unwrap_or(name).to_owned())
        .collect()
}

/// Whether this deck's slide reads differently with each thing taken out. What does not,
/// unless the report names it or what holds it: a report says what is kept by name, so a
/// second `extLst` in a part changes nothing in it and is still said. A kind of thing is
/// tried once for each place it can be, as a part reads slowly in a debug build.
fn silent_in(xml: &[u8]) -> Vec<String> {
    let limits = OoxmlLimits::default();
    let whole = XmlTree::parse(xml, PART, &limits).unwrap();
    let as_read = read(xml).unwrap();
    let named = named_in_report(&as_read);
    let local = |n: &str| n.rsplit(':').next().unwrap_or(n).to_owned();
    let mut silent = Vec::new();
    let mut tried = HashSet::new();
    // Each element with the local names of what holds it, nearest last.
    let mut stack = vec![(whole.root(), Vec::<String>::new())];
    while let Some((parent, above)) = stack.pop() {
        for kid in whole.child_elements(parent) {
            let name = whole.qualified_name(kid).to_owned();
            let attrs = whole.attributes(kid, PART).unwrap();
            let mut path = above.clone();
            path.push(local(&name));
            let kept = path.iter().any(|n| named.contains(n));
            stack.push((kid, path));
            if matches!(name.as_str(), "p:cSld" | "p:spTree" | "p:nvGrpSpPr") || kept {
                continue;
            }
            let held = whole.qualified_name(parent).to_owned();
            let empty = attrs.is_empty() && whole.child_elements(kid).is_empty();
            if !(empty && EMPTY_HOLDERS.contains(&name.as_str()))
                && tried.insert((held.clone(), name.clone(), String::new()))
            {
                let mut tree = whole.clone();
                tree.remove_child(parent, kid);
                if read(&tree.to_bytes()).as_ref() == Some(&as_read) {
                    silent.push(format!("<{name}>"));
                }
            }
            for (attr, _) in attrs {
                let skipped = attr == "xmlns"
                    || attr.starts_with("xmlns:")
                    || NAMELESS.iter().any(|(e, a, _)| *e == name && *a == attr);
                if skipped || !tried.insert((held.clone(), name.clone(), attr.clone())) {
                    continue;
                }
                let mut tree = whole.clone();
                tree.remove_attribute(kid, &attr);
                if read(&tree.to_bytes()).as_ref() == Some(&as_read) {
                    silent.push(format!("<{name}>/@{attr}"));
                }
            }
        }
    }
    silent
}

fn slide_of(deck: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(format!("{deck}.pptx"));
    let bytes = std::fs::read(path).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    pkg.read_part(PART).unwrap()
}

#[test]
fn nothing_in_a_shape_can_go_without_the_model_or_the_report_changing() {
    let mut silent = Vec::new();
    for deck in DECKS {
        let xml = slide_of(deck);
        silent.extend(
            silent_in(&xml)
                .into_iter()
                .map(|what| format!("{deck}: {what}")),
        );
    }
    silent.sort();
    silent.dedup();
    assert!(
        silent.is_empty(),
        "{} read past with nothing said:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// The check itself: a shape with something this importer has never heard of is noticed,
/// and so is taking the thing away, which is what would miss a real drop.
#[test]
fn something_unknown_is_reported_and_so_is_its_absence() {
    let slide = |extra: &str| {
        format!(
            r#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr{extra}/><a:p/></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
        )
    };
    let plain = read(slide("").as_bytes()).unwrap();
    let extra = read(slide(r#" madeUp="1""#).as_bytes()).unwrap();
    assert_ne!(plain, extra, "an unknown attribute changes the report");
    assert!(
        extra.1.iter().any(|f| f.contains("madeUp")),
        "{:?}",
        extra.1
    );
    assert_eq!(
        silent_in(slide(r#" madeUp="1""#).as_bytes()),
        Vec::<String>::new()
    );
}
