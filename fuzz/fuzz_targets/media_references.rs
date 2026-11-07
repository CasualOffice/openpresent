#![no_main]
//! Any bytes as what a picture's `nvPr` and `blip` hold, as an image part's bytes, or as a
//! whole deck. Nothing panics. An unchanged model changes nothing. An edit the exporter
//! accepts reads back as asked and is a fixed point. A catalog of a deck that opens lists
//! parts with hashes that match, finishes within its bounds, and never reads an SVG.

use casual_slide_export::slide::patch_slide_part;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::media::{catalog_media, sha256, sniff_image, MediaLimits};
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{import_slide_part, SlideLimits};
use casual_slide_model::color::Percentage;
use casual_slide_model::fill::RelativeRect;
use casual_slide_model::media::{Media, MediaExtension, MediaFile, MediaTrim};
use casual_slide_model::shape::NodeKind;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/slides/slide1.xml";
const HEAD: &[u8] = br#"<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" xmlns:p173="http://schemas.microsoft.com/office/powerpoint/2017/3/main" xmlns:asvg="http://schemas.microsoft.com/office/drawing/2016/SVG/main" xmlns:woe="http://schemas.microsoft.com/office/word/2020/oembed" xmlns:aif="http://schemas.microsoft.com/office/drawing/2022/imageformula"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:pic><p:nvPicPr><p:cNvPr id="2" name="s"/><p:cNvPicPr/><p:nvPr>"#;
const MIDDLE: &[u8] = br#"</p:nvPr></p:nvPicPr><p:blipFill>"#;
const TAIL: &[u8] = b"</p:blipFill><p:spPr/></p:pic></p:spTree></p:cSld></p:sld>";

fn rels() -> Vec<Relationship> {
    let rel = |id: &str, kind: &str, target: &str, mode| Relationship {
        id: id.to_owned(),
        rel_type: format!(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}"
        ),
        target: target.to_owned(),
        mode,
    };
    vec![
        rel("rId1", "image", "../media/image1.png", TargetMode::Internal),
        rel("rId2", "video", "../media/media1.mp4", TargetMode::Internal),
        rel("rId3", "video", "https://example.invalid/", TargetMode::External),
    ]
}

fn read(
    tree: &XmlTree,
    rels: &[Relationship],
    slide: &SlideLimits,
    import: &ImportLimits,
) -> Option<SlidePart> {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        tree,
        PART,
        rels,
        SlideKind::Slide,
        3,
        slide,
        import,
        &mut findings,
        &mut color,
    )
    .ok()
}

fn slide_and_edits(data: &[u8]) {
    let xml = OoxmlLimits::default();
    let slide = SlideLimits {
        max_shapes_per_part: 2_000,
        max_group_depth: 16,
        ..SlideLimits::default()
    };
    let import = ImportLimits::default();
    let rels = rels();
    // The first half of the input goes in `nvPr`, the rest in `blipFill`.
    let split = data.len() / 2;
    let (app, fill) = data.split_at(split);
    let bytes = [HEAD, app, MIDDLE, fill, TAIL].concat();
    let Ok(tree) = XmlTree::parse(&bytes, PART, &xml) else {
        return;
    };
    let Some(before) = read(&tree, &rels, &slide, &import) else {
        return;
    };

    let mut same = tree.clone();
    let changes =
        patch_slide_part(&mut same, PART, false, &rels, &before, &slide, &import, &xml).unwrap();
    assert_eq!(changes, 0, "an unchanged model changed something");
    assert_eq!(same.to_bytes(), tree.to_bytes());

    let edit = |change: fn(&mut casual_slide_model::shape::ShapeNode)| {
        let mut wanted = before.clone();
        let Some(node) = wanted
            .shape_tree_mut()
            .and_then(|t| t.children.iter_mut().find(|n| n.kind == NodeKind::Picture))
        else {
            return;
        };
        change(node);
        let mut edited = tree.clone();
        let done =
            patch_slide_part(&mut edited, PART, false, &rels, &wanted, &slide, &import, &xml);
        if done.is_ok() {
            let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
            assert_eq!(read(&again, &rels, &slide, &import).as_ref(), Some(&wanted));
            let first = edited.to_bytes();
            let changes =
                patch_slide_part(&mut edited, PART, false, &rels, &wanted, &slide, &import, &xml)
                    .unwrap();
            assert_eq!(changes, 0, "not a fixed point");
            assert_eq!(edited.to_bytes(), first);
        }
    };
    edit(|n| {
        if let Some(p) = &mut n.properties.picture {
            p.source_rect = Some(RelativeRect {
                l: Some(Percentage(1_000)),
                ..RelativeRect::default()
            });
        }
    });
    edit(|n| n.properties.picture = None);
    edit(|n| {
        if let Some(blip) = n.properties.picture.as_mut().and_then(|p| p.blip.as_mut()) {
            blip.embed = Some("rId1".to_owned());
            blip.svg = None;
        }
    });
    edit(|n| n.prefer_relative_resize = Some(false));
    edit(|n| {
        if let Some(app) = &mut n.app {
            app.media = Some(Media::VideoFile(MediaFile {
                link: Some("rId2".to_owned()),
                content_type: None,
            }));
        }
    });
    edit(|n| {
        if let Some(app) = &mut n.app {
            app.media = None;
        }
    });
    edit(|n| {
        if let Some(app) = &mut n.app {
            app.media_extension = Some(Box::new(MediaExtension {
                embed: Some("rId2".to_owned()),
                trim: Some(MediaTrim {
                    st: Some("1".to_owned()),
                    end: None,
                }),
                ..MediaExtension::default()
            }));
        }
    });
    edit(|n| {
        if let Some(app) = &mut n.app {
            app.media_extension = None;
        }
    });
}

fuzz_target!(|data: &[u8]| {
    // An image's first bytes, read in bounds and never trusted.
    let _ = sniff_image(data);
    let hash = sha256(data);
    assert_eq!(hash, sha256(data));

    if data.starts_with(b"PK") {
        let limits = OoxmlLimits {
            max_xml_part_bytes: 1 << 20,
            ..OoxmlLimits::default()
        };
        if let Ok(mut pkg) = PresentationPackage::open(data.to_vec(), limits) {
            let media = MediaLimits {
                max_part_bytes: 1 << 20,
                max_catalog_bytes: 1 << 22,
                ..MediaLimits::default()
            };
            let (catalog, _) = catalog_media(&mut pkg, &media);
            for part in &catalog.parts {
                assert!(!part.referrers.is_empty(), "a part nothing reaches");
                if part.hash.is_some() {
                    let bytes = pkg.read_part(&part.part);
                    if let Ok(bytes) = bytes {
                        assert_eq!(part.hash, Some(sha256(&bytes)));
                    }
                }
            }
        }
        return;
    }
    slide_and_edits(data);
});
