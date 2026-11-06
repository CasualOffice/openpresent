//! Markup compatibility never panics, changes nothing, and gives one answer.
#![no_main]

use casual_slide_ooxml::{resolve_compatibility, OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits {
        max_xml_depth: 64,
        max_xml_elements: 10_000,
        ..OoxmlLimits::default()
    };
    if let Ok(tree) = XmlTree::parse(data, "fuzz", &limits) {
        let before = tree.clone();
        let first = resolve_compatibility(&tree);
        assert_eq!(first, resolve_compatibility(&tree));
        assert_eq!(tree, before);
    }
});
