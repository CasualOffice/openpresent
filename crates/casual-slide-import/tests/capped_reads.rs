//! `PKG-04`: an optional part is read for parsing through the capped read, so one
//! that declares more than the XML walker accepts is refused on its declaration,
//! and none of it is decompressed.

use std::path::{Path, PathBuf};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn an_oversized_optional_part_is_not_decompressed() {
    let source = std::fs::read(fixtures().join("generated/rich-properties.pptx")).unwrap();
    let mut pkg = PresentationPackage::open(source, OoxmlLimits::default()).unwrap();
    // Valid, and large: the size is the only thing wrong with it.
    let padding = "<p:extLst/>".repeat(10_000);
    let big = format!(
        "<p:presentationPr xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">{padding}</p:presentationPr>"
    );
    let mut edits = PartEdits::default();
    edits.replace("ppt/presProps.xml", big.clone().into_bytes());
    let deck = pkg.save_with(&edits).unwrap();

    let limits = OoxmlLimits {
        max_xml_part_bytes: 50_000,
        ..OoxmlLimits::default()
    };
    assert!(big.len() > limits.max_xml_part_bytes);
    let mut pkg = PresentationPackage::open(deck, limits).unwrap();
    let before = pkg.consumed();
    let err = import_deck(&mut pkg, &ImportLimits::default()).unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0009", "{err}");
    let spent = pkg.consumed() - before;
    assert!(
        spent < big.len() as u64,
        "{spent} bytes read; the oversized part was decompressed"
    );
}
