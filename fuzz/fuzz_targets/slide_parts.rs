#![no_main]
//! Any bytes as a slide-like part of every kind, or as a whole deck. Import never
//! panics and stays within its bounds; a patch of an unchanged model changes nothing;
//! an edit the exporter accepts reads back as it was asked; resolving never panics.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, import_slide_parts, SlideLimits};
use casual_slide_model::presentation::{Coordinate, Point};
use casual_slide_model::slide::SlideKind;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_style::Chain;
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const KINDS: [SlideKind; 6] = [
    SlideKind::Slide,
    SlideKind::Layout,
    SlideKind::Master,
    SlideKind::Notes,
    SlideKind::NotesMaster,
    SlideKind::HandoutMaster,
];

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let limits = SlideLimits {
        max_shapes_per_part: 2_000,
        max_group_depth: 16,
        ..SlideLimits::default()
    };
    let import_limits = ImportLimits::default();

    if data.starts_with(b"PK") {
        if let Ok(mut pkg) = PresentationPackage::open(data.to_vec(), xml) {
            if let Ok(deck) = import_deck(&mut pkg, &import_limits) {
                if let Ok(slides) =
                    import_slide_parts(&mut pkg, &deck.deck.presentation, &limits, &import_limits)
                {
                    for o in 0..slides.parts.parts.len() as u32 {
                        let chain = Chain::of(&slides.parts, o);
                        let _ = (chain.color_map(), chain.background(), chain.inherited_shapes());
                        if let Some(tree) = slides.parts.get(o).and_then(|p| p.shape_tree()) {
                            for n in tree.descendants() {
                                let _ = (chain.transform(n), chain.text_style(n));
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    let Ok(tree) = XmlTree::parse(data, PART, &xml) else {
        return;
    };
    for kind in KINDS {
        let mut findings = Findings::new(import_limits.max_findings);
        let mut color = ColorFindings::default();
        let Ok(before) = import_slide_part(
            &tree, PART, &[], kind, 7, &limits, &import_limits, &mut findings, &mut color,
        ) else {
            continue;
        };
        if let Some(t) = before.shape_tree() {
            assert!(t.descendants().count() <= limits.max_shapes_per_part);
        }
        let mut same = tree.clone();
        let changes =
            patch_slide_part(&mut same, PART, false, &[], &before, &limits, &import_limits, &xml)
                .unwrap();
        assert_eq!(changes, 0, "an unchanged model changed something");
        assert_eq!(same.to_bytes(), tree.to_bytes());

        let mut wanted = before.clone();
        if let Some(common) = wanted.common.as_mut() {
            common.name = Some("edited".to_owned());
        }
        wanted.show_master_shapes = Some(false);
        if let Some(node) = wanted
            .shape_tree_mut()
            .and_then(|t| t.children.first_mut())
        {
            if let Some(t) = node.transform.as_mut() {
                t.offset = Some(Point { x: Coordinate(1), y: Coordinate(2) });
                t.rotation = None;
            }
        }
        let mut edited = tree.clone();
        if patch_slide_part(&mut edited, PART, false, &[], &wanted, &limits, &import_limits, &xml)
            .is_ok()
        {
            let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
            let mut f = Findings::new(import_limits.max_findings);
            let mut c = ColorFindings::default();
            let read = import_slide_part(
                &again, PART, &[], kind, 7, &limits, &import_limits, &mut f, &mut c,
            )
            .unwrap();
            assert_eq!(read, wanted);
        }
    }
});
