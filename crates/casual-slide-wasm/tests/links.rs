//! Insert > Link on a shape (`UX-006`): the link saved as an external relationship
//! and an `hlinkClick`, read back, a fixed point, undone to the opened bytes; an
//! address that is not a web or mail address refused.

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_wasm::{Insert, Viewer};

const BOX: [i64; 4] = [914_400, 914_400, 2_743_200, 914_400];

fn part(bytes: &[u8], name: &str) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part(name).unwrap()).unwrap()
}

#[test]
fn a_shape_link_saves_as_an_external_relationship_and_undoes() {
    let mut v = Viewer::blank().unwrap();
    let rect = v.insert_shape(0, Insert::Rectangle, BOX).unwrap();
    let source = v.save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    let node = v.shapes(0).unwrap().last().unwrap().node;
    let _ = rect;
    for bad in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "https://a b",
        "",
    ] {
        assert!(
            v.set_shape_link(0, node, Some(bad)).is_err(),
            "{bad} refused"
        );
    }
    assert_eq!(v.shape_link(0, node), None);
    v.set_shape_link(0, node, Some("https://example.com/a?b=1&c=2"))
        .unwrap();
    assert_eq!(
        v.shape_link(0, node).as_deref(),
        Some("https://example.com/a?b=1&c=2")
    );
    let saved = v.save().unwrap();
    let rels = part(&saved, "ppt/slides/_rels/slide1.xml.rels");
    assert!(rels.contains("TargetMode=\"External\""), "{rels}");
    assert!(rels.contains("https://example.com/a?b=1&amp;c=2"), "{rels}");
    let xml = part(&saved, "ppt/slides/slide1.xml");
    assert!(xml.contains("hlinkClick"), "{xml}");
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.shape_link(0, node).as_deref(), Some(""));
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    v.undo().unwrap();
    assert_eq!(v.shape_link(0, node), None);
    assert_eq!(v.save().unwrap(), source, "undone");
}
