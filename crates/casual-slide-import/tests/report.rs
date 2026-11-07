//! The report names everything not modelled, and a silent drop fails here (`docs/06`, gate 6).
//!
//! The model cannot show what it never saw, so this works from the bytes: put a stranger on an
//! element, and the report must change. A reader that walks past an attribute or a child without
//! a word fails, whichever reader it is. Nothing here knows what any element means.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{ImportFindingKind, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_import::theme::{
    import_theme, import_theme_manager, import_theme_override, read_theme_parts,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_package::PartEdits;

const STRANGER_ATTRIBUTE: &str = " zzUnread=\"1\"";

/// Decks that between them hold every element of the tiers this covers.
const DECKS: &[&str] = &[
    "generated/rich-properties.pptx",
    "generated/package-every-variant.pptx",
    "generated/pres-values-01.pptx",
    "generated/slides-masters-layouts.pptx",
    "generated/shape-every-construct.pptx",
    "generated/theme-every-construct.pptx",
];

/// Elements inside which other sessions' readers answer for what is unread: `CLOSE-01` (drawing
/// and text properties) and `T1G`, `T1H`, `T1I`. A stranger under these is their test's to find.
const READ_ELSEWHERE: &[&str] = &[
    "spPr",
    "grpSpPr",
    "bgPr",
    "bgRef",
    "txBody",
    "txStyles",
    "defaultTextStyle",
    "notesStyle",
    "lstStyle",
    "bodyPr",
    "fmtScheme",
    "objectDefaults",
    "blipFill",
    "graphic",
    "timing",
    "transition",
];

/// Retained as a whole by design: nothing reads inside, and the holder is what is reported.
const RETAINED_WHOLE: &[&str] = &["extLst", "AlternateContent", "custDataLst", "controls"];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

/// Everything the import says about a deck, as one comparable set of lines.
fn report(bytes: &[u8]) -> BTreeSet<String> {
    let mut pkg = open(bytes.to_vec());
    let limits = ImportLimits::default();
    let deck = import_deck(&mut pkg, &limits).unwrap();
    let mut out: BTreeSet<String> = deck.findings.iter().map(|f| format!("{f:?}")).collect();
    out.insert(format!("{:?}", deck.color_findings));
    let slides = import_slide_parts(
        &mut pkg,
        &deck.deck.presentation,
        &SlideLimits::default(),
        &limits,
    )
    .unwrap();
    out.extend(slides.findings.iter().map(|f| format!("{f:?}")));
    out.insert(format!("{:?}", slides.color_findings));
    for (part, tree) in read_theme_parts(&mut pkg).unwrap() {
        match import_theme(&tree) {
            Ok(theme) => out.insert(format!("{part} {:?}", theme.findings)),
            Err(err) => out.insert(format!("{part} {err}")),
        };
    }
    for part in pkg.part_names() {
        let limits = OoxmlLimits::default();
        let tree = |pkg: &mut PresentationPackage| {
            let bytes = pkg.read_xml_part(&part).unwrap();
            XmlTree::parse(&bytes, &part, &limits).unwrap()
        };
        if part.contains("themeOverride") {
            match import_theme_override(&tree(&mut pkg)) {
                Ok(theme) => out.insert(format!("{part} {:?}", theme.findings)),
                Err(err) => out.insert(format!("{part} {err}")),
            };
        } else if part.contains("themeManager") {
            match import_theme_manager(&tree(&mut pkg)) {
                Ok(theme) => out.insert(format!("{part} {:?}", theme.findings)),
                Err(err) => out.insert(format!("{part} {err}")),
            };
        }
    }
    out
}

fn is_read_part(name: &str) -> bool {
    let xml = name.ends_with(".xml") && !name.contains("_rels/");
    xml && (name == "ppt/presentation.xml"
        || name == "ppt/presProps.xml"
        || name == "ppt/viewProps.xml"
        || name.starts_with("docProps/")
        || name.starts_with("ppt/slides/")
        || name.starts_with("ppt/slideLayouts/")
        || name.starts_with("ppt/slideMasters/")
        || name.starts_with("ppt/notesSlides/")
        || name.starts_with("ppt/notesMasters/")
        || name.starts_with("ppt/handoutMasters/")
        || name.starts_with("ppt/theme/theme"))
}

/// One start tag: where its name ends, and whether it is `<x/>`.
struct Tag {
    local: String,
    name_end: usize,
    close_at: usize,
    empty: bool,
}

/// The first start tag of each local name no skipped ancestor holds. The decks here are ours,
/// with no comments or CDATA holding markup.
fn first_tags(text: &str, retained: &BTreeSet<String>) -> Vec<Tag> {
    let mut out: Vec<Tag> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = Vec::new();
    let mut at = 0;
    while let Some(lt) = text[at..].find('<').map(|i| i + at) {
        let close = text[lt..].find('>').map(|i| i + lt).unwrap();
        at = close + 1;
        let body = &text[lt + 1..close];
        if body.starts_with('?') || body.starts_with('!') {
            continue;
        }
        if body.starts_with('/') {
            stack.pop();
            continue;
        }
        let empty = body.ends_with('/');
        let name_len = body
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(body.len());
        let local = body[..name_len].rsplit(':').next().unwrap().to_owned();
        let skip = |n: &String| {
            READ_ELSEWHERE.contains(&n.as_str())
                || RETAINED_WHOLE.contains(&n.as_str())
                || retained.contains(n)
        };
        let hidden = stack.iter().any(skip);
        if !hidden && !skip(&local) && seen.insert(local.clone()) {
            out.push(Tag {
                local: local.clone(),
                name_end: lt + 1 + name_len,
                close_at: close,
                empty,
            });
        }
        if !empty {
            stack.push(local);
        }
    }
    out
}

fn with_part(bytes: &[u8], part: &str, text: String) -> Vec<u8> {
    let mut edits = PartEdits::default();
    edits.replace(part.to_owned(), text.into_bytes());
    open(bytes.to_vec()).save_with(&edits).unwrap()
}

/// Every element read here: an attribute it does not know, and a child it does not know, each
/// change the report. Otherwise the file holds something the user is never told of.
#[test]
fn an_unread_attribute_or_element_is_always_reported() {
    let mut silent = Vec::new();
    let mut checked = 0;
    for deck in DECKS {
        let bytes = std::fs::read(fixtures().join(deck)).unwrap();
        let before = report(&bytes);
        let retained: BTreeSet<String> = {
            let mut pkg = open(bytes.clone());
            let imported = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
            let slides = import_slide_parts(
                &mut pkg,
                &imported.deck.presentation,
                &SlideLimits::default(),
                &ImportLimits::default(),
            )
            .unwrap();
            imported
                .findings
                .iter()
                .chain(slides.findings.iter())
                .filter(|f| f.kind == ImportFindingKind::RetainedElement)
                .map(|f| f.element.clone())
                .collect()
        };
        let mut pkg = open(bytes.clone());
        for part in pkg.part_names() {
            if !is_read_part(&part) {
                continue;
            }
            let Ok(text) = String::from_utf8(pkg.read_part(&part).unwrap()) else {
                continue;
            };
            for tag in first_tags(&text, &retained) {
                checked += 1;
                let mut stranger = text.clone();
                stranger.insert_str(tag.name_end, STRANGER_ATTRIBUTE);
                let named = before
                    .iter()
                    .any(|l| l.contains(&format!("\"{}\"", tag.local)) && l.contains("Retained"));
                if report(&with_part(&bytes, &part, stranger)) == before && !named {
                    silent.push(format!("{deck} {part} {} attribute", tag.local));
                }
                let mut child = text.clone();
                if tag.empty {
                    let at = tag.close_at - 1;
                    child.replace_range(at..=tag.close_at, "><zzUnread/></zzClose>");
                    let name = text[..tag.name_end].rsplit('<').next().unwrap().to_owned();
                    child = child.replace("</zzClose>", &format!("</{name}>"));
                } else {
                    child.insert_str(tag.close_at + 1, "<zzUnread/>");
                }
                if report(&with_part(&bytes, &part, child)) == before {
                    silent.push(format!("{deck} {part} {} child", tag.local));
                }
            }
        }
    }
    assert!(checked > 100, "{checked}");
    assert!(
        silent.is_empty(),
        "{} silent:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// A part no list names is not read, so nothing in it can be reported. The part itself is.
#[test]
fn a_master_no_list_names_is_reported() {
    let bytes = std::fs::read(fixtures().join("generated/package-every-variant.pptx")).unwrap();
    let mut pkg = open(bytes);
    let limits = ImportLimits::default();
    let deck = import_deck(&mut pkg, &limits).unwrap();
    let slides = import_slide_parts(
        &mut pkg,
        &deck.deck.presentation,
        &SlideLimits::default(),
        &limits,
    )
    .unwrap();
    for master in ["notesMaster", "handoutMaster"] {
        assert!(
            slides
                .findings
                .has(ImportFindingKind::RetainedElement, master),
            "{master}"
        );
    }
}
