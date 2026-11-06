//! Every fixture against the expectation recorded for it in `fixtures/manifest.tsv`.

use std::path::{Path, PathBuf};

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn cases() -> Vec<(String, String)> {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .skip(1)
        .map(|line| {
            let mut cols = line.split('\t');
            let path = cols.next().unwrap().to_owned();
            cols.next();
            (path, cols.next().unwrap().to_owned())
        })
        .collect()
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

#[test]
fn every_fixture_meets_its_expectation() {
    let cases = cases();
    assert!(cases.len() > 30, "only {} fixtures", cases.len());
    for (path, expect) in cases {
        let result = PresentationPackage::open(read(&path), OoxmlLimits::default());
        if let Some(code) = expect.strip_prefix("refuse:") {
            match result {
                Ok(_) => panic!("{path}: admitted, expected {code}"),
                Err(err) => assert_eq!(err.code(), code, "{path}: {err}"),
            }
            continue;
        }
        let pkg = result.unwrap_or_else(|err| panic!("{path}: {err}"));
        let mut codes: Vec<&str> = pkg.findings().iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let wanted: Vec<&str> = expect.split(' ').skip(1).collect();
        assert_eq!(codes, wanted, "{path}: {:?}", pkg.findings());
    }
}

fn opening_decks() -> Vec<String> {
    cases()
        .into_iter()
        .filter(|(_, e)| e.starts_with("open"))
        .map(|(p, _)| p)
        .collect()
}

/// Fidelity F1: an unedited save is the file.
#[test]
fn an_unedited_save_is_byte_identical() {
    for path in opening_decks() {
        let bytes = read(&path);
        let pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
        assert_eq!(pkg.save_unedited(), bytes, "{path}");
    }
}

/// Fidelity F2: an edit leaves every other part's bytes alone, and the result
/// still opens with the same graph.
#[test]
fn an_edit_leaves_every_other_part_intact() {
    for path in opening_decks() {
        let mut before = PresentationPackage::open(read(&path), OoxmlLimits::default()).unwrap();
        let edited = before.main_part().to_owned();
        let replacement = before.read_part(&edited).unwrap();
        let mut edits = PartEdits::default();
        edits.replace(edited.clone(), replacement);
        let out = before.save_with(&edits).unwrap();

        let mut after = PresentationPackage::open(out, OoxmlLimits::default()).unwrap();
        assert_eq!(before.part_names(), after.part_names(), "{path}");
        assert_eq!(before.findings(), after.findings(), "{path}");
        for name in before.part_names() {
            assert_eq!(
                before.read_part(&name).unwrap(),
                after.read_part(&name).unwrap(),
                "{path}: {name}"
            );
        }
    }
}

#[test]
fn opening_a_deck_does_not_read_its_media() {
    // Only structure is read on open, so a deck full of media opens cheaply and
    // no decoder is reachable from opening a file.
    let mut pkg =
        PresentationPackage::open(read("generated/image.pptx"), OoxmlLimits::default()).unwrap();
    assert!(pkg.part_names().iter().any(|n| n.ends_with("image1.png")));
    assert!(
        pkg.read_part("PPT/MEDIA/IMAGE1.PNG")
            .unwrap()
            .starts_with(b"\x89PNG")
    );
}

#[test]
fn the_part_graph_finds_a_deck_structure() {
    let pkg =
        PresentationPackage::open(read("generated/three-slides.pptx"), OoxmlLimits::default())
            .unwrap();
    assert_eq!(pkg.main_part(), "ppt/presentation.xml");
    assert_eq!(pkg.relationships_of("ppt/presentation.xml").len(), 5);
    assert!(pkg.findings().is_empty());
    assert_eq!(
        pkg.content_types().resolve("ppt/slides/slide2.xml"),
        Some("application/vnd.openxmlformats-officedocument.presentationml.slide+xml")
    );
}
