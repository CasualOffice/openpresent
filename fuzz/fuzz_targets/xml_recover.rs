//! `REC-03`: the recovering XML reader never panics, and what it writes is what the strict
//! walker admits, under the same bounds. A tree read this way writes back exactly what
//! arrived until it is edited, and writes well-formed text after.
#![no_main]

use casual_slide_ooxml::{recover_xml, walk, OoxmlLimits, Prune, XmlTree};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits {
        max_xml_depth: 64,
        max_xml_elements: 10_000,
        max_xml_attributes: 64,
        ..OoxmlLimits::default()
    };
    for prune in [Prune::Nothing, Prune::SlideLike] {
        let out = recover_xml(data, &limits, prune);
        if out.has_root() {
            walk(&out.bytes, "fuzz", &limits, |_, _| Ok(())).expect("what is recovered is admitted");
            // Reading what was recovered changes nothing: there is nothing left to repair.
            let again = recover_xml(&out.bytes, &limits, Prune::Nothing);
            assert_eq!(again.root_local, out.root_local);
        } else {
            assert!(out.bytes.is_empty());
        }
    }
    let Some((mut tree, _)) = XmlTree::parse_recovering(data, "fuzz", &limits) else {
        return;
    };
    assert_eq!(tree.to_bytes(), data, "untouched, what arrived goes back");
    let root = tree.root();
    if tree.set_attribute(root, "fuzzed", "x") {
        walk(&tree.to_bytes(), "fuzz", &limits, |_, _| Ok(())).expect("an edit is well formed");
    }
});
