//! A checklist row marked as read has to be read: take the construct out of a deck, and the model
//! changes. Reads the marks from `docs/spec/checklist.tsv`, so a false `y` fails here, whatever the
//! source text led the audit to think. By local name: the audit answers for the owner.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_import::theme::{import_theme, import_theme_override, read_theme_parts};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

const AREAS: &[&str] = &[
    "PKG", "SIMPLE", "PRES", "VIEW", "COLOR", "THEME", "SLIDE", "SHAPE",
];
const DECKS: &[&str] = &[
    "generated/rich-properties.pptx",
    "generated/package-every-variant.pptx",
    "generated/pres-values-01.pptx",
    "generated/pres-values-02.pptx",
    "generated/pres-values-03.pptx",
    "generated/slides-masters-layouts.pptx",
    "generated/shape-every-construct.pptx",
    "generated/theme-every-construct.pptx",
    "generated/area-color.pptx",
    "generated/area-slide.pptx",
    "generated/area-shape-tree.pptx",
    "generated/drawing-every-element.pptx",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

/// All the model holds, as text: the deck, its slide-like parts, its themes. No findings.
fn model(bytes: &[u8]) -> String {
    let limits = ImportLimits::default();
    let mut pkg = open(bytes.to_vec());
    let Ok(deck) = import_deck(&mut pkg, &limits) else {
        return "refused".to_owned();
    };
    let mut out = format!("{:?}", deck.deck);
    if let Ok(slides) = import_slide_parts(
        &mut pkg,
        &deck.deck.presentation,
        &SlideLimits::default(),
        &limits,
    ) {
        out += &format!("{:?}", slides.parts);
    }
    if let Ok(parts) = read_theme_parts(&mut pkg) {
        for (name, tree) in parts {
            if name.contains("themeOverride") {
                if let Ok(theme) = import_theme_override(&tree) {
                    out += &format!("{:?}", theme.value);
                }
            } else if let Ok(theme) = import_theme(&tree) {
                out += &format!("{:?}", theme.value);
            }
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
        || name.starts_with("ppt/theme/"))
}

/// One removable thing: its local name, and the bytes it spans.
struct Span {
    name: String,
    start: usize,
    end: usize,
}

/// Every attribute and element in a part we wrote, each of them.
fn spans(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut at = 0;
    while let Some(lt) = text[at..].find('<').map(|i| i + at) {
        let close = text[lt..].find('>').map(|i| i + lt).unwrap();
        at = close + 1;
        let body = &text[lt + 1..close];
        if body.starts_with('?') || body.starts_with('!') {
            continue;
        }
        if body.starts_with('/') {
            let (local, start) = stack.pop().unwrap();
            if stack.is_empty() {
                continue;
            }
            if seen.insert((out.len(), String::new())) {
                out.push(Span {
                    name: local,
                    start,
                    end: at,
                });
            }
            continue;
        }
        let empty = body.ends_with('/');
        let name_len = body
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(body.len());
        let local = body[..name_len].rsplit(':').next().unwrap().to_owned();
        let mut cursor = lt + 1 + name_len;
        while let Some(eq) = text[cursor..close].find("=\"").map(|i| i + cursor) {
            let value_end = eq + 2 + text[eq + 2..close].find('"').unwrap();
            let raw = text[cursor..eq].trim();
            if raw != "xmlns" && !raw.starts_with("xmlns:") {
                let attribute = raw.rsplit(':').next().unwrap().to_owned();
                if seen.insert((out.len(), attribute.clone())) {
                    out.push(Span {
                        name: attribute,
                        start: cursor,
                        end: value_end + 1,
                    });
                }
            }
            cursor = value_end + 1;
        }
        if empty {
            if !stack.is_empty() && seen.insert((out.len(), String::new())) {
                out.push(Span {
                    name: local,
                    start: lt,
                    end: at,
                });
            }
        } else {
            stack.push((local, lt));
        }
    }
    out
}

fn without(bytes: &[u8], part: &str, text: &str, span: &Span) -> Vec<u8> {
    let cut = format!("{}{}", &text[..span.start], &text[span.end..]);
    let mut edits = PartEdits::default();
    edits.replace(part.to_owned(), cut.into_bytes());
    open(bytes.to_vec()).save_with(&edits).unwrap()
}

/// The local names the checklist marks in this column, in the areas this file covers.
fn marked_names() -> BTreeSet<String> {
    let checklist = std::fs::read_to_string(root().join("docs/spec/checklist.tsv")).unwrap();
    let mut header = checklist.lines().next().unwrap().split('\t');
    let at = header.position(|c| c == "I").unwrap();
    let mut names = BTreeSet::new();
    for line in checklist.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if AREAS.contains(&cols[1]) && cols[at] == "y" {
            let name = cols[4].trim_matches(['@', '<', '>']);
            names.insert(name.rsplit(':').next().unwrap().to_owned());
        }
    }
    names
}

#[test]
fn every_row_marked_as_read_changes_the_model_when_taken_out() {
    let mut read = BTreeSet::new();
    let wanted = marked_names();
    for deck in DECKS {
        let bytes = std::fs::read(root().join("fixtures").join(deck)).unwrap();
        let before = model(&bytes);
        let mut pkg = open(bytes.clone());
        for part in pkg.part_names() {
            if !is_read_part(&part) {
                continue;
            }
            let Ok(text) = String::from_utf8(pkg.read_part(&part).unwrap()) else {
                continue;
            };
            for span in spans(&text) {
                if wanted.contains(&span.name)
                    && !read.contains(&span.name)
                    && model(&without(&bytes, &part, &text, &span)) != before
                {
                    read.insert(span.name);
                }
            }
        }
    }
    let checklist = std::fs::read_to_string(root().join("docs/spec/checklist.tsv")).unwrap();
    let mut header = checklist.lines().next().unwrap().split('\t');
    let import_at = header.position(|c| c == "I").unwrap();
    let mut unread = Vec::new();
    for line in checklist.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let (id, area, kind, name) = (cols[0], cols[1], cols[3], cols[4]);
        if AREAS.contains(&area)
            && matches!(kind, "attribute" | "element" | "root")
            && cols[import_at] == "y"
            && !id.starts_with("mc:")
            && !id.starts_with("ct:")
            && !id.starts_with("rel:")
        {
            let local = name
                .trim_matches(['@', '<', '>'])
                .rsplit(':')
                .next()
                .unwrap();
            // A part's root is the part: a deck with none of it reads as a part left out.
            let part_root = kind == "root" && !id.starts_with("vt:");
            if !part_root && !local.starts_with("EG_") && !read.contains(local) {
                unread.push(id.to_owned());
            }
        }
    }
    assert!(read.len() > 100, "{}", read.len());
    assert!(
        unread.is_empty(),
        "{} rows marked as read, and nothing reads them:\n{}",
        unread.len(),
        unread.join("\n")
    );
}
