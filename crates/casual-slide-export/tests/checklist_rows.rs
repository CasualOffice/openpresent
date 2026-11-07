//! A checklist row marked as written has to be written: take the construct out of a deck, ask for
//! the deck as it was, and the save must put it back. Reads the marks from
//! `docs/spec/checklist.tsv`, so a false `y` fails here. By local name: the audit answers for the
//! owner. Theme and colour have setters instead of a model to compare, and their own tests.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use casual_slide_export::presentation::save_deck;
use casual_slide_export::slide::save_slide_parts;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::presentation::Deck;
use casual_slide_model::slide::SlideParts;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

const AREAS: &[&str] = &["PKG", "PRES", "VIEW", "SLIDE", "SHAPE"];
const DECKS: &[&str] = &[
    "generated/rich-properties.pptx",
    "generated/package-every-variant.pptx",
    "generated/pres-values-01.pptx",
    "generated/pres-values-02.pptx",
    "generated/pres-values-03.pptx",
    "generated/slides-masters-layouts.pptx",
    "generated/shape-every-construct.pptx",
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

fn limits() -> ImportLimits {
    ImportLimits::default()
}

fn deck_of(bytes: &[u8]) -> Option<Deck> {
    Some(import_deck(&mut open(bytes.to_vec()), &limits()).ok()?.deck)
}

fn slides_of(bytes: &[u8], deck: &Deck) -> Option<SlideParts> {
    let mut pkg = open(bytes.to_vec());
    let imported = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &limits(),
    );
    Some(imported.ok()?.parts)
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
            if !stack.is_empty() && seen.insert((out.len(), String::new())) {
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

fn is_deck_part(name: &str) -> bool {
    name == "ppt/presentation.xml"
        || name == "ppt/presProps.xml"
        || name == "ppt/viewProps.xml"
        || (name.starts_with("docProps/") && name.ends_with(".xml"))
}

fn is_slide_part(name: &str) -> bool {
    let family = [
        "ppt/slides/",
        "ppt/slideLayouts/",
        "ppt/slideMasters/",
        "ppt/notesSlides/",
        "ppt/notesMasters/",
        "ppt/handoutMasters/",
    ];
    name.ends_with(".xml") && !name.contains("_rels/") && family.iter().any(|f| name.starts_with(f))
}

/// `cut` is `bytes` without the construct. Ask for `bytes`' model over it: is it put back?
fn put_back_in_deck(cut: &[u8], wanted: &Deck) -> bool {
    if deck_of(cut).as_ref() == Some(wanted) {
        return false; // not modelled, so nothing here says whether it is written
    }
    let mut pkg = open(cut.to_vec());
    let Ok(saved) = save_deck(&mut pkg, wanted, &limits()) else {
        return false;
    };
    deck_of(&saved.bytes).as_ref() == Some(wanted)
}

fn put_back_in_slides(cut: &[u8], deck: &Deck, wanted: &SlideParts) -> bool {
    let Some(have) = slides_of(cut, deck) else {
        return false;
    };
    if &have == wanted {
        return false;
    }
    let changed: Vec<_> = wanted
        .parts
        .iter()
        .zip(&have.parts)
        .filter(|(a, b)| a != b)
        .filter_map(|(a, _)| a.as_ref())
        .collect();
    let mut pkg = open(cut.to_vec());
    let Ok(saved) = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &changed,
        &SlideLimits::default(),
        &limits(),
    ) else {
        return false;
    };
    slides_of(&saved.bytes, deck).as_ref() == Some(wanted)
}

/// The local names the checklist marks in this column, in the areas this file covers.
fn marked_names() -> BTreeSet<String> {
    let checklist = std::fs::read_to_string(root().join("docs/spec/checklist.tsv")).unwrap();
    let mut header = checklist.lines().next().unwrap().split('\t');
    let at = header.position(|c| c == "E").unwrap();
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
fn every_row_marked_as_written_comes_back_when_asked_for() {
    let mut written = BTreeSet::new();
    let wanted = marked_names();
    for path in DECKS {
        let bytes = std::fs::read(root().join("fixtures").join(path)).unwrap();
        let Some(deck) = deck_of(&bytes) else {
            continue;
        };
        let slides = slides_of(&bytes, &deck);
        let mut pkg = open(bytes.clone());
        for part in pkg.part_names() {
            let (in_deck, in_slides) = (is_deck_part(&part), is_slide_part(&part));
            if !in_deck && !in_slides {
                continue;
            }
            let Ok(text) = String::from_utf8(pkg.read_part(&part).unwrap()) else {
                continue;
            };
            for span in spans(&text) {
                if written.contains(&span.name) || !wanted.contains(&span.name) {
                    continue;
                }
                let mut edits = PartEdits::default();
                let cut = format!("{}{}", &text[..span.start], &text[span.end..]);
                edits.replace(part.clone(), cut.into_bytes());
                let Ok(cut) = open(bytes.clone()).save_with(&edits) else {
                    continue;
                };
                let back = if in_deck {
                    put_back_in_deck(&cut, &deck)
                } else {
                    slides
                        .as_ref()
                        .is_some_and(|wanted| put_back_in_slides(&cut, &deck, wanted))
                };
                if back {
                    written.insert(span.name);
                }
            }
        }
    }
    let checklist = std::fs::read_to_string(root().join("docs/spec/checklist.tsv")).unwrap();
    let mut header = checklist.lines().next().unwrap().split('\t');
    let export_at = header.position(|c| c == "E").unwrap();
    let mut unwritten = Vec::new();
    for line in checklist.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let (id, area, kind, name) = (cols[0], cols[1], cols[3], cols[4]);
        if AREAS.contains(&area)
            && matches!(kind, "attribute" | "element")
            && cols[export_at] == "y"
            && !id.starts_with("ct:")
            && !id.starts_with("rel:")
        {
            let local = name.trim_start_matches('@').rsplit(':').next().unwrap();
            if !local.starts_with("EG_") && !written.contains(local) {
                unwritten.push(id.to_owned());
            }
        }
    }
    assert!(written.len() > 50, "{}", written.len());
    assert!(
        unwritten.is_empty(),
        "{} rows marked as written, and no save puts them back:\n{}",
        unwritten.len(),
        unwritten.join("\n")
    );
}
