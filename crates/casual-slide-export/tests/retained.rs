//! Tier 1L: a retained object's bytes stay, unedited and through an edit of something else,
//! and its identity cannot be changed through the model.

use std::collections::BTreeMap;
use std::path::Path;

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_part_mapped};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, XmlTree};
use casual_slide_package::PartEdits;

const SLIDE: &str = "ppt/slides/slide1.xml";
const DECKS: [(&str, &[&str]); 4] = [
    (
        "generated/area-ole.pptx",
        &[
            "ppt/embeddings/oleObject1.bin",
            "ppt/embeddings/package1.bin",
            "ppt/activeX/activeX1.xml",
            "ppt/activeX/activeX1.bin",
            "ppt/activeX/_rels/activeX1.xml.rels",
            "ppt/drawings/vmlDrawing1.vml",
            "ppt/media/image1.png",
        ],
    ),
    (
        "generated/area-vml.pptx",
        &[
            "ppt/drawings/vmlDrawing1.vml",
            "ppt/drawings/_rels/vmlDrawing1.vml.rels",
            "ppt/media/image1.png",
        ],
    ),
    (
        "generated/area-model3d.pptx",
        &["ppt/media/model3d1.glb", "ppt/media/image1.png"],
    ),
    (
        "hostile/retained-objects-damaged.bin",
        &[
            "ppt/embeddings/oleObject1.bin",
            "ppt/drawings/vmlDrawing1.vml",
        ],
    ),
];

fn fixture(path: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(path),
    )
    .unwrap()
}

/// The hostile deck's object, its stored bytes changed after its checksum was written.
fn damaged(path: &str) -> Vec<u8> {
    let mut bytes = fixture(path);
    let at = bytes
        .windows(4)
        .position(|w| w == [0xd0, 0xcf, 0x11, 0xe0])
        .unwrap();
    bytes[at] ^= 0xFF;
    bytes
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

/// Each entry's stored bytes and checksum, not what they inflate to: a damaged entry cannot
/// be inflated, and has to come through all the same.
fn stored(zip: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut at = 0;
    while zip.get(at..at + 4) == Some(b"PK\x03\x04") {
        let u16_at = |o: usize| u16::from_le_bytes([zip[at + o], zip[at + o + 1]]) as usize;
        let u32_at = |o: usize| {
            u32::from_le_bytes([
                zip[at + o],
                zip[at + o + 1],
                zip[at + o + 2],
                zip[at + o + 3],
            ]) as usize
        };
        let (name_len, extra) = (u16_at(26), u16_at(28));
        let size = u32_at(18);
        let name = String::from_utf8(zip[at + 30..at + 30 + name_len].to_vec()).unwrap();
        let start = at + 30 + name_len + extra;
        let mut body = zip[at + 14..at + 18].to_vec();
        body.extend_from_slice(&zip[start..start + size]);
        out.insert(name, body);
        at = start + size;
    }
    out
}

fn read_slide(deck: &[u8]) -> (Vec<u8>, XmlTree) {
    let mut package = open(deck.to_vec());
    let xml = package.read_part(SLIDE).unwrap();
    let tree = XmlTree::parse(&xml, SLIDE, &OoxmlLimits::default()).unwrap();
    (xml, tree)
}

fn import(tree: &XmlTree, rels: &[Relationship]) -> SlidePart {
    import_slide_part_mapped(
        tree,
        SLIDE,
        rels,
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .0
}

fn patch(tree: &mut XmlTree, rels: &[Relationship], after: &SlidePart) -> Result<usize, String> {
    patch_slide_part(
        tree,
        SLIDE,
        false,
        rels,
        after,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.to_string())
}

/// Rename the title, which is nothing to do with any retained object, and save.
fn edit_title(deck: &[u8]) -> Vec<u8> {
    let mut package = open(deck.to_vec());
    let rels = package.relationships_of(SLIDE).to_vec();
    let (_, mut tree) = read_slide(deck);
    let mut after = import(&tree, &rels);
    let root = after.common.as_mut().unwrap().shape_tree.as_mut().unwrap();
    let title = &mut root.children[0];
    title.non_visual.as_mut().unwrap().name = Some("Renamed".to_owned());
    patch(&mut tree, &rels, &after).unwrap();
    let mut edits = PartEdits::default();
    edits.replace(SLIDE, tree.to_bytes());
    package.save_with(&edits).unwrap()
}

#[test]
fn an_unedited_save_is_the_file_that_arrived() {
    for (deck, _) in DECKS {
        let bytes = fixture(deck);
        assert_eq!(open(bytes.clone()).save_unedited(), bytes, "{deck}");
    }
}

#[test]
fn an_unrelated_edit_leaves_every_retained_part_byte_identical() {
    for (deck, parts) in DECKS {
        let bytes = fixture(deck);
        let saved = edit_title(&bytes);
        let (a, b) = (stored(&bytes), stored(&saved));
        assert_eq!(a.len(), b.len(), "{deck}: entries");
        for name in parts {
            assert!(
                a.contains_key(*name),
                "{deck}: {name} is not in the fixture"
            );
            assert_eq!(a.get(*name), b.get(*name), "{deck}: {name}");
        }
        let different: Vec<&String> = a.keys().filter(|k| a.get(*k) != b.get(*k)).collect();
        assert_eq!(different, [SLIDE], "{deck}: only the edited part");
    }
}

/// The edit changed the one name: the markup of every object around it is as it was.
#[test]
fn an_unrelated_edit_leaves_the_object_markup_in_the_slide() {
    let bytes = fixture("generated/area-ole.pptx");
    let (xml, _) = read_slide(&bytes);
    let (edited, _) = read_slide(&edit_title(&bytes));
    let (xml, edited) = (
        String::from_utf8(xml).unwrap(),
        String::from_utf8(edited).unwrap(),
    );
    let from = xml.find("<p:graphicFrame>").unwrap();
    let at = edited.find("<p:graphicFrame>").unwrap();
    assert_eq!(&xml[from..], &edited[at..]);
    assert!(xml.contains(r#"name="Title 1""#) && edited.contains(r#"name="Renamed""#));
}

#[test]
fn import_export_import_is_a_fixed_point() {
    for (deck, _) in DECKS {
        let bytes = fixture(deck);
        let rels = open(bytes.clone()).relationships_of(SLIDE).to_vec();
        let (_, tree) = read_slide(&bytes);
        let (_, again) = read_slide(&open(bytes.clone()).save_unedited());
        assert_eq!(import(&tree, &rels), import(&again, &rels), "{deck}");
        // Through an edit, nothing retained moves either.
        let edited = edit_title(&bytes);
        let (_, after) = read_slide(&edited);
        let (a, b) = (import(&tree, &rels), import(&after, &rels));
        let (a, b) = (a.common.unwrap(), b.common.unwrap());
        assert_eq!(a.controls, b.controls, "{deck}");
        let (ta, tb) = (a.shape_tree.unwrap(), b.shape_tree.unwrap());
        assert_eq!(ta.children[1..], tb.children[1..], "{deck}");
    }
}

#[test]
fn changing_an_object_or_a_control_through_the_model_is_refused() {
    let bytes = fixture("generated/area-ole.pptx");
    let rels = open(bytes.clone()).relationships_of(SLIDE).to_vec();
    let (_, mut tree) = read_slide(&bytes);
    let before = import(&tree, &rels);

    let mut after = before.clone();
    let root = after.common.as_mut().unwrap().shape_tree.as_mut().unwrap();
    root.children[2].retained.as_mut().unwrap().prog_id = Some("Other".to_owned());
    let refused = patch(&mut tree, &rels, &after).unwrap_err();
    assert!(refused.contains("a retained object"), "{refused}");

    let mut after = before.clone();
    after.common.as_mut().unwrap().controls.pop();
    let refused = patch(&mut tree, &rels, &after).unwrap_err();
    assert!(refused.contains("a retained control"), "{refused}");
}

#[test]
fn a_damaged_object_comes_through_an_edit_as_it_arrived() {
    let bytes = damaged("hostile/retained-objects-damaged.bin");
    assert_eq!(open(bytes.clone()).save_unedited(), bytes);
    let edited = edit_title(&bytes);
    let (a, b) = (stored(&bytes), stored(&edited));
    let name = "ppt/embeddings/oleObject1.bin";
    // The first four bytes `stored` gives are the checksum. An edit writes a valid file
    // (docs/29), so the bytes come through and the checksum that fails them does not.
    assert_eq!(a[name][4..], b[name][4..]);
    assert_ne!(a[name][..4], b[name][..4]);
    let mut strict = PresentationPackage::open_strict(edited, OoxmlLimits::default()).unwrap();
    assert_eq!(strict.read_part(name).unwrap(), b[name][4..]);
}
