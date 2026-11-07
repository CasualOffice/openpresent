#![no_main]
//! Any bytes as a part of a diagram, standalone OMML, or an ink part. Import never panics;
//! the parts' agreement can always be asked for; a patch of an unchanged model changes
//! nothing; a part emptied and written from its model reads as that model, or is refused.

use casual_slide_export::diagram::patch_diagram_part;
use casual_slide_export::ink::patch_ink_part;
use casual_slide_export::math::patch_math_part;
use casual_slide_import::diagram::{import_diagram_root, GraphicLimits};
use casual_slide_import::ink::import_ink_root;
use casual_slide_import::math::import_math_root;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_model::diagram::schema::Root;
use casual_slide_model::diagram::Diagram;
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/diagrams/part.xml";

fn blank(tree: &XmlTree) -> XmlTree {
    let mut out = tree.clone();
    let root = out.root();
    for kid in out.child_elements(root) {
        out.remove_child(root, kid);
    }
    for (name, _) in out.declared_attributes(root) {
        if name != "xmlns" && !name.starts_with("xmlns:") {
            out.remove_attribute(root, &name);
        }
    }
    out
}

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let Ok(tree) = XmlTree::parse(data, PART, &xml) else {
        return;
    };
    let (limits, import) = (GraphicLimits::default(), ImportLimits::default());
    let mut findings = Findings::new(import.max_findings);
    if let Ok(Some(root)) = import_diagram_root(&tree, PART, &[], &limits, &import, &mut findings) {
        let mut diagram = Diagram::default();
        match &root {
            Root::DataModel(v) => diagram.data = Some((**v).clone()),
            Root::LayoutDef(v) => diagram.layout = Some((**v).clone()),
            Root::DspDrawing(v) => diagram.drawing = Some((**v).clone()),
            _ => {}
        }
        let _ = diagram.inconsistencies();
        let _ = diagram.drawing_shapes();
        let _ = diagram.set_point_text("1", "x");
        let mut same = tree.clone();
        let changed = patch_diagram_part(&mut same, PART, false, &[], &root, &limits, &import, &xml);
        assert!(matches!(changed, Ok(0)), "an unchanged model changed the part");
        let _ = patch_diagram_part(&mut blank(&tree), PART, false, &[], &root, &limits, &import, &xml);
    }
    if let Ok(Some(root)) = import_math_root(&tree, PART, &[], &limits, &import, &mut findings) {
        let mut same = tree.clone();
        let changed = patch_math_part(&mut same, PART, false, &[], &root, &limits, &import, &xml);
        assert!(matches!(changed, Ok(0)), "an unchanged model changed the part");
        let _ = patch_math_part(&mut blank(&tree), PART, false, &[], &root, &limits, &import, &xml);
    }
    if let Ok(Some(root)) = import_ink_root(&tree, PART, &[], &limits, &import, &mut findings) {
        let mut same = tree.clone();
        let changed = patch_ink_part(&mut same, PART, false, &[], &root, &limits, &import, &xml);
        assert!(matches!(changed, Ok(0)), "an unchanged model changed the part");
        let _ = patch_ink_part(&mut blank(&tree), PART, false, &[], &root, &limits, &import, &xml);
    }
});
