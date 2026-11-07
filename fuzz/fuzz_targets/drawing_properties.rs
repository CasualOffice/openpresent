#![no_main]
//! Any bytes as the inside of a shape's properties, or as a whole slide part. Import
//! never panics and stays within its bounds; a patch of an unchanged model changes
//! nothing; an edit of a fill, a line, effects and 3D that the exporter accepts reads
//! back as it was asked, and saving it again is a fixed point; a custom geometry that
//! read compiles or is refused, and never panics.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::geometry::compile_custom;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, SlideLimits};
use casual_slide_model::color::{Color, Percentage};
use casual_slide_model::d3::Scene3D;
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::{Fill, GradientFill, GradientStop};
use casual_slide_model::geometry::Geometry;
use casual_slide_model::line::{Line, LineDash, LineJoin};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const HEAD: &[u8] = br#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a14="http://schemas.microsoft.com/office/drawing/2010/main" xmlns:ask="http://schemas.microsoft.com/office/drawing/2018/sketchyshapes"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>"#;
const TAIL: &[u8] = b"</p:spPr></p:sp></p:spTree></p:cSld></p:sld>";

fn read(tree: &XmlTree, slide: &SlideLimits, import: &ImportLimits) -> Option<SlidePart> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree,
        PART,
        &[],
        SlideKind::Slide,
        3,
        slide,
        import,
        &mut findings,
        &mut color,
    )
    .ok()
}

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let slide = SlideLimits {
        max_shapes_per_part: 2_000,
        max_group_depth: 16,
        ..SlideLimits::default()
    };
    let import = ImportLimits::default();

    let wrapped;
    let bytes = if data.first() == Some(&b'<') && data.starts_with(b"<p:sld") {
        data
    } else {
        wrapped = [HEAD, data, TAIL].concat();
        &wrapped
    };
    let Ok(tree) = XmlTree::parse(bytes, PART, &xml) else {
        return;
    };
    let Some(before) = read(&tree, &slide, &import) else {
        return;
    };

    // Nothing edited: nothing changed.
    let mut same = tree.clone();
    let changes =
        patch_slide_part(&mut same, PART, false, &[], &before, &slide, &import, &xml).unwrap();
    assert_eq!(changes, 0, "an unchanged model changed something");
    assert_eq!(same.to_bytes(), tree.to_bytes());

    let Some(root) = before.shape_tree() else {
        return;
    };
    for node in root.descendants() {
        if let Some(Geometry::Custom(g)) = &node.properties.geometry {
            let _ = compile_custom(g);
        }
    }

    // An edit either is refused, leaving what was asked unsaid, or reads back as asked.
    let edit = |change: fn(&mut casual_slide_model::drawing::ShapeProperties)| {
        let mut wanted = before.clone();
        if let Some(node) = wanted.shape_tree_mut().and_then(|t| t.children.first_mut()) {
            change(&mut node.properties);
        } else {
            return;
        }
        let mut edited = tree.clone();
        let done =
            patch_slide_part(&mut edited, PART, false, &[], &wanted, &slide, &import, &xml);
        if done.is_ok() {
            let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
            assert_eq!(read(&again, &slide, &import).as_ref(), Some(&wanted));
            let first = edited.to_bytes();
            let changes =
                patch_slide_part(&mut edited, PART, false, &[], &wanted, &slide, &import, &xml)
                    .unwrap();
            assert_eq!(changes, 0, "not a fixed point");
            assert_eq!(edited.to_bytes(), first);
        }
    };
    edit(|p| p.fill = Some(Fill::Solid(Some(Color::srgb(1, 2, 3)))));
    edit(|p| {
        p.fill = Some(Fill::Gradient(Box::new(GradientFill {
            stops: Some(vec![GradientStop {
                position: Some(Percentage(1_000)),
                color: Some(Color::srgb(4, 5, 6)),
            }]),
            ..GradientFill::default()
        })))
    });
    edit(|p| p.fill = None);
    edit(|p| {
        let line = p.line.get_or_insert_with(Box::<Line>::default);
        line.width = Some(100);
        line.dash = Some(LineDash::Custom(Vec::new()));
        line.join = Some(LineJoin::Round);
        line.sketch = None;
    });
    edit(|p| p.line = None);
    edit(|p| p.effects = Some(EffectProperties::List(Vec::new())));
    edit(|p| p.scene3d = Some(Box::new(Scene3D::default())));
    edit(|p| p.sp3d = None);
    edit(|p| p.geometry = None);
    edit(|p| p.hidden = None);
});
