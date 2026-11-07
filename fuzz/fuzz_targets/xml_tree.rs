//! `FUZZ-01`: the tree the editor keeps never panics, agrees with the walker about
//! what is XML, writes back exactly what it read, and reads every attribute of a
//! part it admitted. An edit through it leaves a part that still parses.
#![no_main]

use casual_slide_ooxml::{walk, OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits {
        max_xml_depth: 64,
        max_xml_elements: 10_000,
        max_xml_attributes: 64,
        ..OoxmlLimits::default()
    };
    let walked = walk(data, "fuzz", &limits, |_, _| Ok(()));
    let tree = XmlTree::parse(data, "fuzz", &limits);
    // One decision about what is XML (`XML-02`, `XML-03`).
    assert_eq!(walked.is_ok(), tree.is_ok(), "{walked:?} {tree:?}");
    if let Err(err) = &tree {
        assert_eq!(err.code(), walked.unwrap_err().code());
        return;
    }
    let Ok(mut tree) = tree else { return };
    assert_eq!(tree.to_bytes(), data);

    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        let attributes = tree.attributes(id, "fuzz").expect("the walker admitted it");
        assert!(tree.attributes_valid(id));
        for (name, value) in &attributes {
            assert_eq!(tree.plain_attribute(id, name).as_deref(), Some(value.as_str()));
        }
        let _ = tree.namespace(id);
        let _ = tree.own_text(id);
        stack.extend(tree.child_elements(id));
    }

    let root = tree.root();
    tree.set_text(root, "x < y & \"z\"\r\n");
    if tree.set_attribute(root, "fuzzed", "a'\"<&\t") {
        let again = XmlTree::parse(&tree.to_bytes(), "fuzz", &limits)
            .expect("an edited part parses");
        assert_eq!(again.own_text(again.root()).as_deref(), Some("x < y & \"z\"\r\n"));
        assert_eq!(again.plain_attribute(again.root(), "fuzzed").as_deref(), Some("a'\"<&\t"));
    }
});
