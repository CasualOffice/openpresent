//! `SEC-018`: no document text in any import error or finding, as `SEC-015` for the
//! layers below.
//!
//! A marker is planted wherever a part can hold text of its own: element, attribute
//! and prefix names, values, text and targets. Then every code in `docs/20` for this
//! crate is produced, and neither the message nor the debug form of what comes back
//! may hold the marker. The codes themselves are checked, so a code that stops being
//! produced fails here rather than passing unseen.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::{import_deck, import_presentation};
use casual_slide_import::properties::{
    Findings, ImportLimits, import_app_properties, import_core_properties,
};
use casual_slide_import::slide::{LazySlides, SlideLimits, import_slide_parts, index_deck};
use casual_slide_import::theme::import_theme;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree};
use casual_slide_package::PartEdits;

const MARK: &str = "QzMarkerQz";
const PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const DML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const RELNS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const T: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const PRES_RELS: &str = "ppt/_rels/presentation.xml.rels";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn with(parts: &[(&str, String)]) -> PresentationPackage {
    let bytes = std::fs::read(fixtures().join("generated/minimal.pptx")).unwrap();
    let mut pkg = PresentationPackage::open_strict(bytes, OoxmlLimits::default()).unwrap();
    let mut edits = PartEdits::default();
    for (name, xml) in parts {
        edits.replace(*name, xml.clone().into_bytes());
    }
    let saved = pkg.save_with(&edits).unwrap();
    PresentationPackage::open_strict(saved, OoxmlLimits::default()).unwrap()
}

/// The minimal deck's main relationships, and these after them.
fn pres_rels(extra: &[(&str, &str, &str)]) -> String {
    let mut out = format!(
        "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"{T}/slideMaster\" Target=\"slideMasters/slideMaster1.xml\"/>\
         <Relationship Id=\"rId2\" Type=\"{T}/theme\" Target=\"theme/theme1.xml\"/>\
         <Relationship Id=\"rId3\" Type=\"{T}/slide\" Target=\"slides/slide1.xml\"/>"
    );
    for (id, kind, target) in extra {
        out.push_str(&format!(
            "<Relationship Id=\"{id}\" Type=\"{T}/{kind}\" Target=\"{target}\"/>"
        ));
    }
    out + "</Relationships>"
}

/// A main part with these children around the minimal deck's lists.
fn main_part(root_attrs: &str, children: &str) -> String {
    format!(
        "<p:presentation xmlns:p=\"{PML}\" xmlns:r=\"{RELNS}\" xmlns:x=\"urn:{MARK}\" {root_attrs}>\
         <p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rId1\"/></p:sldMasterIdLst>\
         <p:sldIdLst><p:sldId id=\"256\" r:id=\"rId3\"/></p:sldIdLst>\
         <p:sldSz cx=\"12192000\" cy=\"6858000\"/><p:notesSz cx=\"6858000\" cy=\"9144000\"/>{children}\
         </p:presentation>"
    )
}

fn rel(id: &str, kind: &str) -> Relationship {
    Relationship {
        id: id.to_owned(),
        rel_type: format!("{T}/{kind}"),
        target: format!("{id}.xml"),
        mode: TargetMode::Internal,
    }
}

#[derive(Default)]
struct Seen {
    codes: BTreeSet<String>,
}

impl Seen {
    fn clean(&mut self, what: &str, text: String) {
        assert!(!text.contains(MARK), "{what} carries document text: {text}");
    }

    fn findings(&mut self, what: &str, findings: &Findings) {
        self.clean(what, format!("{findings:?}"));
        for f in findings.iter() {
            self.codes.insert(f.kind.code().to_owned());
        }
    }

    fn color_findings(&mut self, what: &str, findings: &ColorFindings) {
        self.clean(what, format!("{findings:?}"));
        for code in findings.codes() {
            self.codes.insert(code.to_owned());
        }
    }

    fn error<E: std::fmt::Debug + std::fmt::Display>(&mut self, what: &str, code: &str, err: E) {
        self.clean(what, format!("{err} {err:?}"));
        self.codes.insert(code.to_owned());
    }
}

/// Each case is a main part, read with the given relationships.
fn presentation_cases(seen: &mut Seen) {
    let limits = ImportLimits::default();
    let rels = [rel("rId1", "slideMaster"), rel("rId7", "theme")];
    for (what, xml) in [
        (
            "a value that does not parse",
            format!(
                "<p:presentation xmlns:p=\"{PML}\" firstSlideNum=\"{MARK}\" autoCompressPictures=\"{MARK}\"/>"
            ),
        ),
        (
            "ids out of range and repeated, and a missing r:id",
            format!(
                "<p:presentation xmlns:p=\"{PML}\" xmlns:r=\"{RELNS}\"><p:sldIdLst>\
                 <p:sldId id=\"1\" r:id=\"rId7\"/><p:sldId id=\"300\"/>\
                 <p:sldId id=\"301\" r:id=\"{MARK}\"/><p:sldId id=\"301\" r:id=\"rId7\"/>\
                 </p:sldIdLst></p:presentation>"
            ),
        ),
        (
            "a repeated element",
            format!(
                "<p:presentation xmlns:p=\"{PML}\"><p:sldSz cx=\"914400\" cy=\"914400\"/><p:sldSz cx=\"914400\" cy=\"914400\"/></p:presentation>"
            ),
        ),
        (
            "names nobody models",
            format!("<x:{MARK} xmlns:x=\"urn:x\"/>"),
        ),
        (
            "an unknown attribute and element, by name and by prefix",
            format!(
                "<p:presentation xmlns:p=\"{PML}\" xmlns:{MARK}=\"urn:u\" {MARK}:{MARK}=\"{MARK}\" {MARK}=\"{MARK}\">\
                 <p:{MARK}/><{MARK}:{MARK}/></p:presentation>"
            ),
        ),
    ] {
        let tree = XmlTree::parse(xml.as_bytes(), "part", &OoxmlLimits::default()).unwrap();
        let mut findings = Findings::new(100);
        match import_presentation(&tree, "part", &rels, &limits, &mut findings) {
            Ok(_) => {}
            Err(err) => seen.error(what, err.code(), err),
        }
        seen.findings(what, &findings);
    }
}

fn properties_cases(seen: &mut Seen) {
    let limits = ImportLimits::default();
    let app = format!(
        "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\" \
         xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">\
         <HeadingPairs><vt:vector size=\"9\" baseType=\"lpstr\"><vt:lpstr>{MARK}</vt:lpstr></vt:vector></HeadingPairs>\
         <TotalTime>{MARK}</TotalTime></Properties>"
    );
    let tree = XmlTree::parse(app.as_bytes(), "app", &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::new(100);
    let _ = import_app_properties(&tree, "app", &limits, &mut findings);
    seen.findings("app properties", &findings);

    let small = ImportLimits {
        max_keywords: 0,
        max_value_bytes: 4,
        ..ImportLimits::default()
    };
    for core in [
        format!(
            "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\">\
             <cp:keywords xml:lang=\"en\"><cp:value>{MARK}</cp:value></cp:keywords></cp:coreProperties>"
        ),
        format!(
            "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\">\
             <cp:category>{MARK}</cp:category></cp:coreProperties>"
        ),
        format!("<{MARK}/>"),
    ] {
        let tree = XmlTree::parse(core.as_bytes(), "core", &OoxmlLimits::default()).unwrap();
        let mut findings = Findings::new(100);
        match import_core_properties(&tree, "core", &small, &mut findings) {
            Ok(_) => {}
            Err(err) => seen.error("core properties", err.code(), err),
        }
        seen.findings("core properties", &findings);
    }
}

fn deck_cases(seen: &mut Seen) {
    let limits = ImportLimits::default();
    let unreadable = with(&[
        (
            PRES_RELS,
            pres_rels(&[("rId9", "presProps", "presProps.xml")]),
        ),
        ("ppt/presProps.xml", format!("<{MARK}/>")),
    ]);
    let dup = with(&[
        (
            PRES_RELS,
            pres_rels(&[
                ("rId8", "presProps", "presProps.xml"),
                ("rId9", "presProps", "presProps.xml"),
            ]),
        ),
        (
            "ppt/presProps.xml",
            format!("<p:presentationPr xmlns:p=\"{PML}\"/>"),
        ),
    ]);
    let unusable = with(&[(
        PRES_RELS,
        pres_rels(&[("rId9", "presProps", "../../../up.xml")]),
    )]);
    let show = with(&[
        (
            PRES_RELS,
            pres_rels(&[("rId9", "slide", "slides/slide1.xml")]),
        ),
        (
            "ppt/presentation.xml",
            main_part(
                "",
                &format!(
                    "<p:custShowLst><p:custShow name=\"{MARK}\" id=\"1\"><p:sldLst>\
                     <p:sld r:id=\"rId9\"/></p:sldLst></p:custShow></p:custShowLst>"
                ),
            ),
        ),
    ]);
    for (what, mut pkg) in [
        ("an unreadable part", unreadable),
        ("two relationships for one role", dup),
        ("a relationship leaving the package", unusable),
        ("a custom show naming no slide of the deck", show),
    ] {
        match import_deck(&mut pkg, &limits) {
            Ok(imported) => seen.findings(what, &imported.findings),
            Err(err) => seen.error(what, err.code(), err),
        }
    }

    // A report that reached its bound says so.
    let small = ImportLimits {
        max_findings: 1,
        ..ImportLimits::default()
    };
    let mut crowded = with(&[(
        "ppt/presentation.xml",
        main_part(
            &format!("x:{MARK}=\"1\" x:other=\"2\" x:third=\"3\" firstSlideNum=\"{MARK}\""),
            "",
        ),
    )]);
    let imported = import_deck(&mut crowded, &small).unwrap();
    seen.findings("a truncated report", &imported.findings);
}

fn theme_cases(seen: &mut Seen) {
    let many = "<a:alpha val=\"50000\"/>".repeat(65);
    let list =
        format!("<a:custClr name=\"{MARK}\"><a:srgbClr val=\"010203\"/></a:custClr>").repeat(4097);
    let xml = format!(
        "<a:theme xmlns:a=\"{DML}\" name=\"{MARK}\"><a:themeElements>\
         <a:clrScheme name=\"{MARK}\">\
         <a:dk1><a:srgbClr val=\"{MARK}\"/></a:dk1>\
         <a:lt1><a:srgbClr val=\"010203\">{many}</a:srgbClr></a:lt1>\
         <a:dk2><a:srgbClr val=\"010203\"><a:alpha val=\"{MARK}\"/></a:srgbClr></a:dk2>\
         <a:lt2><a:srgbClr val=\"010203\"><a:alpha val=\"33.33333%\"/></a:srgbClr></a:lt2>\
         <a:accent1><a:srgbClr val=\"010203\"/><a:srgbClr val=\"040506\"/></a:accent1>\
         <a:accent2><a:{MARK}/></a:accent2>\
         <a:accent3><a:{MARK} val=\"{MARK}\"/><{MARK} xmlns=\"urn:{MARK}\"/></a:accent3>\
         </a:clrScheme>\
         <a:fontScheme name=\"{MARK}\"><a:majorFont><a:latin typeface=\"{MARK}\" panose=\"{MARK}\"/>\
         <a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:majorFont></a:fontScheme>\
         </a:themeElements><a:{MARK}/><a:custClrLst>{list}</a:custClrLst></a:theme>"
    );
    let tree = XmlTree::parse(xml.as_bytes(), "theme", &OoxmlLimits::default()).unwrap();
    let got = import_theme(&tree).unwrap();
    seen.color_findings("a theme", &got.findings);

    let no_elements = format!("<a:theme xmlns:a=\"{DML}\"/>");
    let tree = XmlTree::parse(no_elements.as_bytes(), "theme", &OoxmlLimits::default()).unwrap();
    seen.color_findings("a theme", &import_theme(&tree).unwrap().findings);

    let wrong = format!("<{MARK}/>");
    let tree = XmlTree::parse(wrong.as_bytes(), "theme", &OoxmlLimits::default()).unwrap();
    if let Err(err) = import_theme(&tree) {
        seen.clean("a wrong theme root", format!("{err} {err:?}"));
    }
}

fn slide_cases(seen: &mut Seen) {
    let limits = ImportLimits::default();
    let nested = format!(
        "<p:sld xmlns:p=\"{PML}\" xmlns:a=\"{DML}\" {MARK}=\"{MARK}\"><p:cSld name=\"{MARK}\"><p:spTree>\
         <p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"{MARK}\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>\
         <p:grpSp><p:nvGrpSpPr><p:cNvPr id=\"2\" name=\"{MARK}\" descr=\"{MARK}\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:grpSp>\
         <p:{MARK}/></p:spTree></p:cSld></p:sld>"
    );
    let open = || with(&[("ppt/slides/slide1.xml", nested.clone())]);
    let run = |seen: &mut Seen, what: &str, slides: SlideLimits| {
        let mut pkg = open();
        let deck = import_deck(&mut pkg, &limits).unwrap().deck;
        match import_slide_parts(&mut pkg, &deck.presentation, &slides, &limits) {
            Ok(got) => {
                seen.findings(what, &got.findings);
                seen.color_findings(what, &got.color_findings);
            }
            Err(err) => seen.error(what, err.code(), err),
        }
    };
    run(seen, "plain", SlideLimits::default());
    for (what, slides) in [
        (
            "too many shapes",
            SlideLimits {
                max_shapes_per_part: 1,
                ..SlideLimits::default()
            },
        ),
        (
            "groups too deep",
            SlideLimits {
                max_group_depth: 0,
                ..SlideLimits::default()
            },
        ),
        (
            "too many slides",
            SlideLimits {
                max_slides: 0,
                ..SlideLimits::default()
            },
        ),
        (
            "too many parts",
            SlideLimits {
                max_parts: 0,
                ..SlideLimits::default()
            },
        ),
    ] {
        let mut pkg = open();
        let deck = import_deck(&mut pkg, &limits).unwrap().deck;
        let mut findings = Findings::new(100);
        let result = index_deck(&pkg, &deck.presentation, &slides, &mut findings);
        seen.findings(what, &findings);
        match result {
            Err(err) => seen.error(what, err.code(), err),
            Ok(_) => run(seen, what, slides),
        }
    }

    let mut pkg = open();
    let deck = import_deck(&mut pkg, &limits).unwrap().deck;
    let mut findings = Findings::new(100);
    let structure = index_deck(
        &pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &mut findings,
    )
    .unwrap();
    let mut lazy = LazySlides::new(structure, SlideLimits::default(), limits);
    let err = lazy.part(&mut pkg, 999_999).err().unwrap();
    seen.error("no such part", err.code(), err);
}

#[test]
fn no_error_or_finding_carries_document_text() {
    let mut seen = Seen::default();
    presentation_cases(&mut seen);
    properties_cases(&mut seen);
    deck_cases(&mut seen);
    theme_cases(&mut seen);
    slide_cases(&mut seen);

    // CS-CLR-0004 is absent on purpose: the tree refuses malformed attributes first, so
    // no file reaches it, and it stays as the reader's own check (`XML-02`).
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    for (prefix, count) in [("CS-IMF", 15), ("CS-IMP", 3), ("CS-CLR", 10), ("CS-SLD", 5)] {
        for n in 1..=count {
            wanted.insert(format!("{prefix}-{n:04}"));
        }
    }
    wanted.remove("CS-CLR-0004");
    let missing: Vec<_> = wanted.difference(&seen.codes).collect();
    assert!(missing.is_empty(), "never produced: {missing:?}");
}

/// What a name a part chose looks like in a report: known names stay, others do not.
#[test]
fn a_schema_name_is_kept_and_any_other_name_is_not() {
    let xml = format!(
        "<p:presentation xmlns:p=\"{PML}\" xmlns:x=\"urn:x\" x:{MARK}=\"1\" bogusAttribute=\"1\" rtl=\"{MARK}\">\
         <p:notAnElement/><p:{MARK}/></p:presentation>"
    );
    let tree = XmlTree::parse(xml.as_bytes(), "part", &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::new(100);
    let limits = ImportLimits::default();
    import_presentation(&tree, "part", &[], &limits, &mut findings).unwrap();
    let text = format!("{findings:?}");
    assert!(!text.contains(MARK), "{text}");
    assert!(!text.contains("notAnElement"), "{text}");
    assert!(!text.contains("bogusAttribute"), "{text}");
    assert!(text.contains("\"presentation\""), "{text}");
    assert!(text.contains("\"rtl\""), "{text}");
}
