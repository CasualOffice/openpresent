#![no_main]
//! Any bytes as what a shape's `p:txBody` holds, and as a list style in a presentation's
//! `defaultTextStyle`. Import never panics; a patch of an unchanged model changes nothing;
//! an edit the exporter accepts reads back as it was asked; resolving never panics.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_presentation;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, SlideLimits};
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::text::{CharacterProperties, TextItem, TextRun};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use casual_slide_style::Chain;
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#
);

fn read(tree: &XmlTree, limits: &SlideLimits, import: &ImportLimits) -> Option<SlidePart> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree, PART, &[], SlideKind::Slide, 0, limits, import, &mut findings, &mut color,
    )
    .ok()
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let xml = OoxmlLimits::default();
    let limits = SlideLimits::default();
    let import = ImportLimits::default();

    let slide = format!(
        r#"<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr><p:ph idx="1"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody>{text}</p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
    );
    if let Ok(tree) = XmlTree::parse(slide.as_bytes(), PART, &xml) {
        if let Some(before) = read(&tree, &limits, &import) {
            let mut same = tree.clone();
            let changes =
                patch_slide_part(&mut same, PART, false, &[], &before, &limits, &import, &xml)
                    .unwrap();
            assert_eq!(changes, 0, "an unchanged model changed something");
            assert_eq!(same.to_bytes(), tree.to_bytes());

            // Resolve everything the body holds, through a chain with this shape at
            // every level, and each with the others' default.
            if let Some(node) = before.shape_tree().and_then(|t| t.children.first()) {
                let chain = Chain {
                    slide: Some(&before),
                    layout: None,
                    master: None,
                };
                let text_chain = chain.text_chain(node, node.text_body.as_ref().and_then(|b| b.list_style.as_ref()));
                if let Some(body) = &node.text_body {
                    let _ = text_chain.body(body.properties.as_ref(), |b| b.anchor);
                    for p in &body.paragraphs {
                        let own = p.properties.as_deref();
                        let _ = (text_chain.margin_left(own), text_chain.default_tab_size(own));
                        for item in &p.items {
                            if let TextItem::Run(r) = item {
                                let _ = text_chain.font_size(r.properties.as_deref(), own);
                                let _ = text_chain.run_fill(r.properties.as_deref(), own);
                            }
                        }
                    }
                }
            }

            // An edit: every run's text and size, a new run at each paragraph's end.
            let mut wanted = before.clone();
            if let Some(body) = wanted
                .shape_tree_mut()
                .and_then(|t| t.children.first_mut())
                .and_then(|n| n.text_body.as_mut())
            {
                for p in &mut body.paragraphs {
                    for item in &mut p.items {
                        if let TextItem::Run(r) = item {
                            r.text = Some("edited & <new>".to_owned());
                            r.properties
                                .get_or_insert_with(Box::<CharacterProperties>::default)
                                .size = Some(1234);
                        }
                    }
                    p.items.push(TextItem::Run(TextRun {
                        properties: None,
                        text: Some(String::new()),
                    }));
                }
            }
            let mut edited = tree.clone();
            if patch_slide_part(&mut edited, PART, false, &[], &wanted, &limits, &import, &xml)
                .is_ok()
            {
                let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
                assert_eq!(read(&again, &limits, &import).as_ref(), Some(&wanted));
            }
        }
    }

    let presentation = format!(
        r#"<p:presentation {NS}><p:defaultTextStyle>{text}</p:defaultTextStyle></p:presentation>"#
    );
    if let Ok(tree) = XmlTree::parse(presentation.as_bytes(), "ppt/presentation.xml", &xml) {
        let mut findings = Findings::new(import.max_findings);
        let _ = import_presentation(&tree, "ppt/presentation.xml", &[], &import, &mut findings);
    }
});
