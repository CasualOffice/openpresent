//! Parts that are not XML: a caller's ceiling on reading one, and the Microsoft
//! relationship types that reach audio, video and captions (`MS-PPTX`).

use std::path::Path;

use casual_slide_ooxml::{FindingKind, OoxmlLimits, PresentationPackage};

fn open(rel: &str) -> PresentationPackage {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel);
    PresentationPackage::open(std::fs::read(path).unwrap(), OoxmlLimits::default()).unwrap()
}

#[test]
fn a_binary_part_over_the_ceiling_is_refused_with_nothing_inflated() {
    let mut pkg = open("generated/media-every-reference.pptx");
    let name = "ppt/media/image1.png";
    let size = u64::try_from(pkg.read_part(name).unwrap().len()).unwrap();
    let before = pkg.consumed();
    let err = pkg.read_part_capped(name, size - 1).unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0009");
    assert_eq!(pkg.consumed(), before, "nothing was charged");
    assert_eq!(
        u64::try_from(pkg.read_part_capped(name, size).unwrap().len()).unwrap(),
        size,
        "at the ceiling it reads"
    );
    // Found by any equivalent spelling, as `read_part` is, and absent is its own refusal.
    assert!(pkg.read_part_capped("PPT/MEDIA/IMAGE1.PNG", size).is_ok());
    assert_eq!(
        pkg.read_part_capped("ppt/media/none.png", size)
            .unwrap_err()
            .code(),
        "CS-OPC-0001"
    );
}

#[test]
fn microsoft_media_and_track_relationships_are_known() {
    let pkg = open("generated/media-every-reference.pptx");
    let slide = "ppt/slides/slide1.xml";
    let rels = pkg.relationships_of(slide);
    for tail in ["/relationships/media", "/relationships/track"] {
        assert!(rels.iter().any(|r| r.rel_type.ends_with(tail)), "{tail}");
    }
    assert!(
        !pkg.findings()
            .iter()
            .any(|f| f.kind == FindingKind::UnknownRelationshipType),
        "{:?}",
        pkg.findings()
    );
}

/// `REL-01`: the relationship types of Microsoft's that import follows and reads, chartEx
/// with its style and colours, a diagram's drawing, and modern comments with their authors,
/// are not reported as unknown. A healthy deck holding them says nothing about them.
#[test]
fn microsoft_chart_and_comment_relationships_that_import_reads_are_known() {
    const READ: [&str; 6] = [
        "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing",
        "http://schemas.microsoft.com/office/2014/relationships/chartEx",
        "http://schemas.microsoft.com/office/2011/relationships/chartStyle",
        "http://schemas.microsoft.com/office/2011/relationships/chartColorStyle",
        "http://schemas.microsoft.com/office/2018/10/relationships/comments",
        "http://schemas.microsoft.com/office/2018/10/relationships/authors",
    ];
    let mut seen = std::collections::BTreeSet::new();
    // Real decks: chartEx, style and colours; comment authors; a diagram's drawing.
    for deck in [
        "real/sdk-of16-01.pptx",
        "real/z14230715-ddi-ko.pptx",
        "real/z20089741-tasamoh-ar.pptx",
        "generated/comments-modern.pptx",
    ] {
        let pkg = open(deck);
        let mut sources = pkg.part_names();
        sources.push(String::new());
        for source in sources {
            for rel in pkg.relationships_of(&source) {
                if !READ.contains(&rel.rel_type.as_str()) {
                    continue;
                }
                seen.insert(rel.rel_type.clone());
                assert!(
                    !pkg.findings().iter().any(|f| {
                        f.kind == FindingKind::UnknownRelationshipType
                            && f.part == source
                            && f.rel_id.as_deref() == Some(rel.id.as_str())
                    }),
                    "{deck}: {source} {} reported unknown",
                    rel.rel_type
                );
            }
        }
    }
    assert_eq!(seen.len(), READ.len(), "each type is in a deck: {seen:?}");
}
