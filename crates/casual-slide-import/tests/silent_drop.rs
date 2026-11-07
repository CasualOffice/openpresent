//! A silent drop fails here (`docs/06`, per-tier gate 6). For every element and attribute
//! of every shape in the decks that hold the most of Tiers 1E and 1F: take it out, read the
//! part again, and something must differ, in the model or in the report. If neither
//! differs, the importer read past it and said nothing, which is the one thing it may not
//! do. Slide parts only, and only inside the shape tree; other tiers keep their own.

use std::collections::HashSet;
use std::path::Path;

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

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

fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// Every element in the tree, with what holds it.
fn elements(tree: &XmlTree) -> Vec<(ElementId, ElementId)> {
    let mut out = Vec::new();
    let mut stack = vec![tree.root()];
    while let Some(parent) = stack.pop() {
        for kid in tree.child_elements(parent) {
            out.push((parent, kid));
            stack.push(kid);
        }
    }
    out
}

/// Whether taking this element out goes unnoticed. A report says what is kept by name, so a
/// second `extLst` in a part changes nothing in it: every other element of the name goes
/// first, and then this one has to be what the report was saying. An element inside one the
/// report names is that element's: `ext` in `extLst`, or `a:r` in an `mc:Fallback`. Whichever
/// holds it is asked this same question in its own turn.
fn goes_unnoticed(whole: &XmlTree, parent: ElementId, kid: ElementId, above: &[String]) -> bool {
    let without = |tree: &XmlTree| {
        let mut gone = tree.clone();
        gone.remove_child(parent, kid);
        read(&gone.to_bytes())
    };
    let as_read = read(&whole.to_bytes());
    if without(whole) != as_read {
        return false;
    }
    let name = local(whole.qualified_name(kid)).to_owned();
    let mut alone = whole.clone();
    let others = elements(whole);
    let inside = |candidate: ElementId| {
        let mut stack = vec![candidate];
        while let Some(id) = stack.pop() {
            if id == kid {
                return true;
            }
            stack.extend(whole.child_elements(id));
        }
        false
    };
    for (holder, other) in others {
        if other != kid && local(whole.qualified_name(other)) == name && !inside(other) {
            alone.remove_child(holder, other);
        }
    }
    let Some(isolated) = read(&alone.to_bytes()) else {
        return false;
    };
    if without(&alone).as_ref() != Some(&isolated) {
        return false;
    }
    !named_above(&isolated, above)
}

/// What in this slide reads no differently for being gone. A kind of thing is tried once for
/// each place it can be, as a part reads slowly in a debug build.
fn silent_in(xml: &[u8]) -> Vec<String> {
    let whole = XmlTree::parse(xml, PART, &OoxmlLimits::default()).unwrap();
    let as_read = read(xml).unwrap();
    let mut silent = Vec::new();
    let mut tried = HashSet::new();
    // Each element with the local names of what holds it.
    let mut stack = vec![(whole.root(), Vec::<String>::new())];
    while let Some((parent, above)) = stack.pop() {
        for kid in whole.child_elements(parent) {
            let name = whole.qualified_name(kid).to_owned();
            let attrs = whole.attributes(kid, PART).unwrap();
            let mut path = above.clone();
            path.push(local(&name).to_owned());
            stack.push((kid, path));
            if matches!(name.as_str(), "p:cSld" | "p:spTree" | "p:nvGrpSpPr") {
                continue;
            }
            let held = whole.qualified_name(parent).to_owned();
            let empty = attrs.is_empty() && whole.child_elements(kid).is_empty();
            if !(empty && EMPTY_HOLDERS.contains(&name.as_str()))
                && tried.insert((held.clone(), name.clone(), String::new()))
                && goes_unnoticed(&whole, parent, kid, &above)
            {
                silent.push(format!("<{name}> in <{held}>"));
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
                if read(&tree.to_bytes()).as_ref() == Some(&as_read)
                    && !named_above(&as_read, &above)
                {
                    silent.push(format!("<{name}>/@{attr} in <{held}>"));
                }
            }
        }
    }
    silent
}

/// Whether the report names something holding this.
fn named_above((_, report, colors): &Read, above: &[String]) -> bool {
    report
        .iter()
        .chain(colors)
        .filter_map(|line| line.split(' ').nth(1))
        .any(|named| above.iter().any(|a| a == local(named)))
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
        extra.1.iter().any(|f| f.contains("bodyPr")),
        "{:?}",
        extra.1
    );
    assert_eq!(
        silent_in(slide(r#" madeUp="1""#).as_bytes()),
        Vec::<String>::new()
    );
}
