#![no_main]
//! Any bytes as what `p:timing` holds, as what `p:transition` holds, and in a branch of
//! `mc:AlternateContent`. Import never panics, and a slide always reads; a patch of an
//! unchanged model changes nothing; taking timing and transition away and asking for them
//! back reads back as asked; a target naming no shape never saves; an edit the exporter
//! accepts reads back as it was asked.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, SlideLimits};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::timing::{Slot, TimeNode, TimeValue};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" "#,
    r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" "#,
    r#"xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" "#,
    r#"xmlns:p15="http://schemas.microsoft.com/office/powerpoint/2012/main" "#,
    r#"xmlns:p159="http://schemas.microsoft.com/office/powerpoint/2015/09/main""#
);
const TREE: &str = r#"<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp></p:spTree></p:cSld>"#;

fn read(tree: &XmlTree, limits: &SlideLimits, import: &ImportLimits) -> Option<SlidePart> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree, PART, &[], SlideKind::Slide, 0, limits, import, &mut findings, &mut color,
    )
    .ok()
}

fn patched(
    tree: &XmlTree,
    wanted: &SlidePart,
    limits: &SlideLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Option<XmlTree> {
    let mut edited = tree.clone();
    patch_slide_part(&mut edited, PART, false, &[], wanted, limits, import, xml).ok()?;
    let again = XmlTree::parse(&edited.to_bytes(), PART, xml).unwrap();
    assert_eq!(read(&again, limits, import).as_ref(), Some(wanted));
    Some(edited)
}

fn try_one(slide: &str, limits: &SlideLimits, import: &ImportLimits, xml: &OoxmlLimits) {
    let Ok(tree) = XmlTree::parse(slide.as_bytes(), PART, xml) else {
        return;
    };
    // Whatever timing holds, that the XML parsed means the part reads.
    let before = read(&tree, limits, import).expect("bad timing lost the slide");

    let mut same = tree.clone();
    let changes =
        patch_slide_part(&mut same, PART, false, &[], &before, limits, import, xml).unwrap();
    assert_eq!(changes, 0, "an unchanged model changed something");
    assert_eq!(same.to_bytes(), tree.to_bytes());
    let _ = before.animated_spids();

    // Away and back, which writes all of it from the model. Refusal is allowed, as a tree
    // that reads can hold what cannot be made; what saves must read back as asked.
    let mut gone = before.clone();
    gone.timing = None;
    gone.transition = None;
    if let Some(stripped) = patched(&tree, &gone, limits, import, xml) {
        let _ = patched(&stripped, &before, limits, import, xml);
    }

    // An edit to every root of the tree.
    let mut wanted = before.clone();
    if let Some(Slot::Plain(timing)) = &mut wanted.timing {
        for node in timing.tn_lst.iter_mut().flatten() {
            if let TimeNode::Par(par) = node {
                par.common.get_or_insert_with(Default::default).dur =
                    Some(TimeValue::Milliseconds(5));
            }
        }
    }
    let _ = patched(&tree, &wanted, limits, import, xml);

    // A shape that moves has its targets move with it, and no target is left dangling.
    let mut moved = before.clone();
    if moved.retarget_shape(2, 3) > 0 {
        let _ = patched(&tree, &moved, limits, import, xml);
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let xml = OoxmlLimits::default();
    let limits = SlideLimits::default();
    let import = ImportLimits::default();
    for slide in [
        format!(r#"<p:sld {NS}>{TREE}<p:timing>{text}</p:timing></p:sld>"#),
        format!(r#"<p:sld {NS}>{TREE}<p:transition>{text}</p:transition></p:sld>"#),
        format!(
            r#"<p:sld {NS}>{TREE}<mc:AlternateContent><mc:Choice Requires="p14"><p:timing>{text}</p:timing></mc:Choice><mc:Fallback><p:timing/></mc:Fallback></mc:AlternateContent></p:sld>"#
        ),
    ] {
        try_one(&slide, &limits, &import, &xml);
    }
});
