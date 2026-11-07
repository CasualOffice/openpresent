//! `REC-02`, `REC-04`, `REC-05`: any bytes open or are refused only by `docs/29`'s rules. The
//! original comes back from an unedited save, slides are there to read, and an edit writes a
//! file that opens under `strict`.
#![no_main]

use casual_slide_export::presentation::save_deck;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{import_slide_parts, SlideLimits};
use casual_slide_model::properties::CoreProperties;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PackageLimits;
use libfuzzer_sys::fuzz_target;

/// Real decks, damaged by the input: a seed is the deck, the rest the bytes to lay over it.
const DECKS: [&[u8]; 3] = [
    include_bytes!("../../fixtures/generated/minimal.pptx"),
    include_bytes!("../../fixtures/generated/three-slides.pptx"),
    include_bytes!("../../fixtures/generated/slides-strict.pptx"),
];

fn deck_from(data: &[u8]) -> Vec<u8> {
    if data.starts_with(b"PK") {
        return data.to_vec();
    }
    let Some((choice, rest)) = data.split_first() else {
        return Vec::new();
    };
    let mut deck = DECKS[usize::from(*choice) % DECKS.len()].to_vec();
    for chunk in rest.chunks(3) {
        if let [hi, lo, value] = chunk {
            let at = usize::from(*hi) << 8 | usize::from(*lo);
            let len = deck.len();
            if let Some(byte) = deck.get_mut(at % len.max(1)) {
                *byte = *value;
            }
        }
    }
    deck
}

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits {
        package: PackageLimits {
            max_total_uncompressed: 64 << 20,
            max_entries: 4_096,
            ..PackageLimits::default()
        },
        max_xml_elements: 100_000,
        ..OoxmlLimits::default()
    };
    let import_limits = ImportLimits::default();
    let deck = deck_from(data);
    let mut package = match PresentationPackage::open(deck.clone(), limits) {
        Ok(package) => package,
        Err(err) => {
            assert!(
                matches!(err.code(), "CS-PKG-0005" | "CS-PKG-0001" | "CS-OPC-0005"),
                "refused outside docs/29: {err}"
            );
            return;
        }
    };
    assert_eq!(package.save_unedited(), deck, "an unedited save is the input");
    let imported = import_deck(&mut package, &import_limits).expect("a deck that opened reads");
    let slides = import_slide_parts(
        &mut package,
        &imported.deck.presentation,
        &SlideLimits::default(),
        &import_limits,
    )
    .expect("slides that opened read");
    let _ = slides.damaged.len();
    let _ = package.recovery_report().plain_language();
    assert_eq!(package.save_unedited(), deck, "reading changes nothing");

    let mut after = imported.deck.clone();
    after
        .properties
        .core
        .get_or_insert_with(CoreProperties::default)
        .title = Some("edited".to_owned());
    let edited = save_deck(&mut package, &after, &import_limits).expect("an edit is written");
    PresentationPackage::open_strict(edited.bytes, limits).expect("the output is clean");
});
