//! Every text body on every slide of the generated and real decks lays out, and no
//! character a bundled face can draw goes missing on the way.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::text::{TextBody, TextItem};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_style::Chain;
use casual_slide_text::{Frame, TextEnv, layout_text_body};

fn decks(folder: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(folder);
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "pptx"))
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

/// Characters a drawn layout must hold: every non-white-space one a bundled or
/// supplied face covers, upper-cased so `cap="all"` compares equal.
fn counts(text: &str, only_drawable: bool) -> BTreeMap<char, usize> {
    let mut out = BTreeMap::new();
    for ch in text.chars().flat_map(char::to_uppercase) {
        // CJK is drawn from slices a host registers, and none is registered here.
        if ch.is_whitespace()
            || (only_drawable
                && (casual_slide_text::cjk::is_cjk(ch)
                    || casual_slide_text::faces::coverage_face_bytes(ch, false, false).is_none()))
        {
            continue;
        }
        *out.entry(ch).or_insert(0) += 1;
    }
    out
}

fn body_text(body: &TextBody) -> String {
    let mut text = String::new();
    for p in &body.paragraphs {
        for item in &p.items {
            match item {
                TextItem::Run(r) => text.push_str(r.text.as_deref().unwrap_or("")),
                TextItem::Field(f) => text.push_str(f.text.as_deref().unwrap_or("")),
                _ => {}
            }
        }
    }
    text
}

struct Totals {
    bodies: usize,
    runs: usize,
}

fn lay_out_deck(path: &Path, totals: &mut Totals) {
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let Ok(mut pkg) = PresentationPackage::open(bytes, OoxmlLimits::default()) else {
        return;
    };
    let Ok(deck) = import_deck(&mut pkg, &ImportLimits::default()) else {
        return;
    };
    let deck = deck.deck;
    let Ok(parts) = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    ) else {
        return;
    };
    let parts = parts.parts;
    let default = deck.presentation.default_text_style.as_ref();
    let frame = Frame {
        x: 0.0,
        y: 0.0,
        width: 720.0,
        height: 540.0,
    };
    for index in 0..10_000 {
        let Some(ordinal) = parts.structure.slide(index) else {
            break;
        };
        let Some(slide) = parts.get(ordinal) else {
            continue;
        };
        let chain = Chain::of(&parts, ordinal);
        let Some(tree) = slide.shape_tree() else {
            continue;
        };
        for shape in tree.descendants() {
            let Some(body) = shape.text_body.as_deref() else {
                continue;
            };
            let text_chain = chain.text_chain(shape, default, None);
            let layout = layout_text_body(body, &text_chain, &TextEnv::default(), frame);
            totals.bodies += 1;
            totals.runs += layout.runs.len();
            for run in &layout.runs {
                assert!(!run.glyphs.is_empty(), "{name}: an empty run");
                assert!(run.size > 0.0, "{name}: size {}", run.size);
                for g in &run.glyphs {
                    assert!(g.x.is_finite() && g.y.is_finite(), "{name}: glyph at {g:?}");
                }
                assert!(
                    casual_slide_text::face_by_key(&run.face_key).is_some(),
                    "{name}: face {} cannot be found again",
                    run.face_key
                );
            }
            let drawn: String = layout.runs.iter().map(|r| r.text.as_str()).collect();
            let drawn = counts(&drawn, false);
            for (ch, wanted) in counts(&body_text(body), true) {
                let got = drawn.get(&ch).copied().unwrap_or(0);
                assert!(
                    got >= wanted,
                    "{name}, slide {index}: U+{:04X} drawn {got} times of {wanted}",
                    u32::from(ch)
                );
            }
        }
    }
}

#[test]
fn every_text_body_in_the_generated_and_real_decks_lays_out_without_losing_text() {
    let mut totals = Totals { bodies: 0, runs: 0 };
    let all: Vec<PathBuf> = decks("generated")
        .into_iter()
        .chain(decks("real"))
        .collect();
    assert!(all.len() > 100, "the corpus is where it should be");
    for path in &all {
        lay_out_deck(path, &mut totals);
    }
    assert!(totals.bodies > 100, "{} bodies", totals.bodies);
    assert!(totals.runs > 100, "{} runs", totals.runs);
}
