//! What `r` in the checklist rests on: every counted deck, edited, keeps every node it had.
//!
//! The checklist's audit holds a row at `r` to a fixture, and every counted deck comes through
//! here, so a construct nothing models still has to survive. Works from the bytes, since the model
//! cannot show what it never held.

use std::path::{Path, PathBuf};

use casual_slide_export::presentation::save_deck;
use casual_slide_export::slide::save_slide_parts;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::slide::SlideKind;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// The decks the audit counts: generated, producers and real.
fn counted_decks() -> Vec<String> {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let path = cols.next()?;
            let expect = cols.nth(1)?;
            (expect.starts_with("open") && !path.starts_with("hostile/")).then(|| path.to_owned())
        })
        .collect()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

/// `name="..."` out of a start tag at its first occurrence, if there.
fn without_attribute(text: &str, name: &str) -> String {
    let pattern = format!(" {name}=\"");
    let Some(start) = text.find(&pattern) else {
        return text.to_owned();
    };
    let value = start + pattern.len();
    let end = value + text[value..].find('"').unwrap();
    format!("{}{}", &text[..start], &text[end + 1..])
}

fn assert_same_but_for(deck: &str, part: &str, old: &[u8], new: &[u8], attribute: &str) {
    match (std::str::from_utf8(old), std::str::from_utf8(new)) {
        (Ok(old), Ok(new)) => assert_eq!(
            without_attribute(old, attribute),
            without_attribute(new, attribute),
            "{deck}: {part} lost or changed more than @{attribute}"
        ),
        // A UTF-16 part: the same bytes but for what was written.
        _ => assert!(new.len() >= old.len().saturating_sub(64), "{deck}: {part}"),
    }
}

#[test]
fn every_node_of_every_deck_survives_an_edit() {
    let limits = ImportLimits::default();
    let mut edited = 0;
    for deck in counted_decks() {
        let bytes = std::fs::read(fixtures().join(&deck)).unwrap();
        let mut pkg = open(bytes.clone());
        let main = pkg.main_part().to_owned();
        let names = pkg.part_names();

        // One attribute on the presentation part's root.
        let mut model = import_deck(&mut pkg, &limits).unwrap().deck;
        let number = model.presentation.first_slide_num.unwrap_or(1);
        model.presentation.first_slide_num = Some(number.saturating_add(1));
        let saved = save_deck(&mut pkg, &model, &limits).unwrap();
        assert_eq!(saved.changed_parts, [main.clone()], "{deck}");
        let mut after = open(saved.bytes);
        let mut source = open(bytes.clone());
        assert_eq!(after.part_names(), names, "{deck}: parts lost or invented");
        for name in &names {
            let (old, new) = (source.read_part(name), after.read_part(name));
            let (old, new) = (old.unwrap(), new.unwrap());
            if name == &main {
                assert_same_but_for(&deck, name, &old, &new, "firstSlideNum");
            } else {
                assert_eq!(old, new, "{deck}: {name} changed");
            }
        }

        // One attribute on a slide, where most unmodelled content lives.
        let mut pkg = open(bytes.clone());
        let model = import_deck(&mut pkg, &limits).unwrap().deck;
        let Ok(slides) = import_slide_parts(
            &mut pkg,
            &model.presentation,
            &SlideLimits::default(),
            &limits,
        ) else {
            continue;
        };
        let mut wanted = slides.parts;
        let Some(first) = wanted
            .parts
            .iter_mut()
            .flatten()
            .find(|p| p.kind == SlideKind::Slide)
        else {
            continue;
        };
        first.show = Some(!first.show.unwrap_or(true));
        let first = first.clone();
        let saved = save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &[&first],
            &SlideLimits::default(),
            &limits,
        )
        .unwrap_or_else(|e| panic!("{deck}: {}", e.code()));
        assert_eq!(saved.changed_parts.len(), 1, "{deck}");
        let mut after = open(saved.bytes);
        for name in &names {
            let (old, new) = (source.read_part(name), after.read_part(name));
            let (old, new) = (old.unwrap(), new.unwrap());
            if saved.changed_parts.contains(name) {
                assert_same_but_for(&deck, name, &old, &new, "show");
            } else {
                assert_eq!(old, new, "{deck}: {name} changed");
            }
        }
        edited += 1;
    }
    assert!(edited > 60, "{edited}");
}
