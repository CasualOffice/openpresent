//! A silent drop fails here (`docs/06`, per-tier gate 6). For every element and attribute
//! of every shape in the decks that hold the most of Tiers 1E and 1F: take it out, read the
//! part again, and something must differ, in the model or in the report. If neither
//! differs, the importer read past it and said nothing, which is the one thing it may not
//! do. Slide parts inside the shape tree, and theme parts (`T1C-03`); other tiers keep their own.

use std::collections::HashSet;
use std::path::Path;

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part};
use casual_slide_import::theme::import_theme;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

const PART: &str = "ppt/slides/slide1.xml";
/// The decks walked. Anything with a body-level `extLst` or a foreign child is here too, as
/// retention must be reported as much as anything else.
const DECKS: [&str; 8] = [
    "drawing-every-element",
    "drawing-extension-lists",
    "text-every-property",
    "text-warp-and-3d",
    "text-object-defaults",
    "shape-fills",
    "area-ole",
    "area-model3d",
];
/// Read on purpose by something other than its text, each with why.
const NAMELESS: [(&str, &str, &str); 1] = [
    // An extension is matched by what it holds: real decks spell one `uri` many ways.
    ("a:ext", "uri", "CS-DRW docs/22"),
];

/// Required by the schema and holding nothing when empty: taking one away loses nothing, so
/// there is nothing to notice. Each is still kept by the tree.
const EMPTY_HOLDERS: [&str; 7] = [
    "p:spPr",
    "p:cNvSpPr",
    "p:cNvGrpSpPr",
    "p:cNvPicPr",
    "p:cNvCxnSpPr",
    "p:cNvGraphicFramePr",
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

const RICH_THEME: &str = include_str!("data/rich-theme.xml");
const THEME_DECKS: [&str; 3] = ["text-object-defaults", "area-text", "shape-fills"];

fn read_theme(xml: &[u8]) -> Option<(String, Vec<String>)> {
    let tree = XmlTree::parse(xml, "t", &OoxmlLimits::default()).ok()?;
    let imported = import_theme(&tree).ok()?;
    let report = imported
        .findings
        .items
        .iter()
        .map(|f| format!("{} {} {}", f.kind.code(), f.element, f.detail))
        .collect();
    Some((format!("{:?}", imported.value), report))
}

/// What in a theme reads no differently for being gone. What an extension holds is its own
/// matter: its `uri` is in the model, and the rest stays in the part.
fn silent_in_theme(xml: &[u8]) -> Vec<String> {
    let whole = XmlTree::parse(xml, "t", &OoxmlLimits::default()).unwrap();
    let as_read = read_theme(xml).unwrap();
    let mut silent = Vec::new();
    let mut stack = vec![whole.root()];
    while let Some(parent) = stack.pop() {
        if local(whole.qualified_name(parent)) == "ext" {
            continue;
        }
        for kid in whole.child_elements(parent) {
            stack.push(kid);
            let name = whole.qualified_name(kid).to_owned();
            let mut gone = whole.clone();
            gone.remove_child(parent, kid);
            if read_theme(&gone.to_bytes()).as_ref() == Some(&as_read) {
                silent.push(format!("<{name}>"));
            }
            for (attr, _) in whole.attributes(kid, "t").unwrap() {
                if attr == "xmlns" || attr.starts_with("xmlns:") {
                    continue;
                }
                let mut gone = whole.clone();
                gone.remove_attribute(kid, &attr);
                if read_theme(&gone.to_bytes()).as_ref() == Some(&as_read) {
                    silent.push(format!("<{name}>/@{attr}"));
                }
            }
        }
    }
    silent.sort();
    silent.dedup();
    silent
}

/// `T1C-03`.
#[test]
fn nothing_in_a_theme_can_go_without_the_model_or_the_report_changing() {
    assert_eq!(
        silent_in_theme(RICH_THEME.as_bytes()),
        Vec::<String>::new(),
        "rich theme"
    );
    for deck in THEME_DECKS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/generated")
            .join(format!("{deck}.pptx"));
        let bytes = std::fs::read(path).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        let xml = pkg.read_part("ppt/theme/theme1.xml").unwrap();
        assert_eq!(silent_in_theme(&xml), Vec::<String>::new(), "{deck}");
    }
}

/// `T1C-03`: what the importer has never heard of, on anything a format scheme or an object
/// default is made of, is reported under that element's name.
#[test]
fn a_stranger_on_a_format_scheme_or_object_default_is_reported_where_it_is() {
    const A: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"";
    let body = |marks: [&str; 14]| {
        let [
            fmt,
            fills,
            lines,
            ln,
            effects,
            style,
            bg,
            defaults,
            sp,
            pr,
            bp,
            ls,
            st,
            tail,
        ] = marks;
        format!(
            r#"<a:theme {A}><a:themeElements><a:fmtScheme name="f"{fmt}>
<a:fillStyleLst{fills}><a:noFill/><a:noFill/><a:noFill/></a:fillStyleLst>
<a:lnStyleLst{lines}><a:ln{ln}/><a:ln/><a:ln/></a:lnStyleLst>
<a:effectStyleLst{effects}><a:effectStyle{style}/><a:effectStyle/><a:effectStyle/></a:effectStyleLst>
<a:bgFillStyleLst{bg}><a:noFill/><a:noFill/><a:noFill/></a:bgFillStyleLst>
</a:fmtScheme></a:themeElements><a:objectDefaults{defaults}><a:spDef{sp}><a:spPr{pr}/><a:bodyPr{bp}/><a:lstStyle{ls}/><a:style{st}/></a:spDef>{tail}</a:objectDefaults></a:theme>"#
        )
    };
    let plain = read_theme(body([""; 14]).as_bytes()).unwrap();
    let names = [
        "fmtScheme",
        "fillStyleLst",
        "lnStyleLst",
        "ln",
        "effectStyleLst",
        "effectStyle",
        "bgFillStyleLst",
        "objectDefaults",
        "spDef",
        "spPr",
        "bodyPr",
        "lstStyle",
        "style",
    ];
    for (at, name) in names.iter().enumerate() {
        let mut marks = [""; 14];
        marks[at] = r#" madeUp="1""#;
        let read = read_theme(body(marks).as_bytes()).unwrap();
        assert_eq!(read.0, plain.0, "{name}: the model does not change");
        assert_ne!(read.1, plain.1, "{name}: an unknown attribute is reported");
    }
}

/// `T1C-03`: `xfrm` and `@bwMode` in a default's `spPr` have no place in the model, so they
/// have to be in the report.
#[test]
fn what_a_default_sp_pr_holds_beyond_the_model_is_reported() {
    let xml = RICH_THEME.replacen(
        "<a:spPr><a:solidFill>",
        r#"<a:spPr bwMode="auto"><a:xfrm/><a:solidFill>"#,
        1,
    );
    assert_ne!(xml, RICH_THEME);
    let plain = read_theme(RICH_THEME.as_bytes()).unwrap();
    let read = read_theme(xml.as_bytes()).unwrap();
    assert_eq!(read.0, plain.0);
    let added: Vec<&String> = read.1.iter().filter(|f| !plain.1.contains(f)).collect();
    assert!(added.iter().any(|f| f.contains("xfrm")), "{added:?}");
    assert!(added.iter().any(|f| f.contains("spPr")), "{added:?}");
}
