//! Tier 1D import: slides, layouts, masters, notes, placeholders and shape trees.

use std::path::{Path, PathBuf};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{
    ImportedSlides, LazySlides, SlideError, SlideLimits, import_slide_parts, index_deck,
};
use casual_slide_model::presentation::{Coordinate, Direction, Point, PositiveSize};
use casual_slide_model::shape::{
    Action, BlackWhiteMode, Connection, NodeKind, ObjectId, PlaceholderSize, PlaceholderType,
    ShapeNode,
};
use casual_slide_model::slide::{BackgroundFill, SlideKind, SlideLayoutType, SlidePart};
use casual_slide_model::theme::{ColorMapOverride, ColorMapSlot, ColorSchemeIndex};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn slides_of(bytes: Vec<u8>) -> ImportedSlides {
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
}

fn slides(path: &str) -> ImportedSlides {
    slides_of(std::fs::read(fixtures().join(path)).unwrap())
}

#[test]
fn every_generated_and_producer_deck_imports() {
    let mut count = 0;
    for set in ["generated", "producers", "real"] {
        for entry in std::fs::read_dir(fixtures().join(set)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "tsv" || e == "md") {
                continue;
            }
            let rel = format!("{set}/{}", path.file_name().unwrap().to_string_lossy());
            let s = slides(&rel);
            let st = &s.parts.structure;
            assert!(!st.slides.is_empty(), "{rel}: no slides");
            assert!(!st.masters.is_empty(), "{rel}: no master");
            for (i, part) in s.parts.parts.iter().enumerate() {
                assert!(part.is_some(), "{rel}: part {i} unread: {:?}", s.findings);
            }
            assert!(
                !s.findings.has(K::UnreadablePart, ""),
                "{rel}: {:?}",
                s.findings
            );
            count += 1;
        }
    }
    assert!(count >= 30, "only {count} decks");
}

const RICH: &str = "generated/slides-masters-layouts.pptx";

fn open(path: &str) -> PresentationPackage {
    PresentationPackage::open(
        std::fs::read(fixtures().join(path)).unwrap(),
        OoxmlLimits::default(),
    )
    .unwrap()
}

fn try_slides(path: &str, limits: &SlideLimits) -> Result<ImportedSlides, SlideError> {
    let mut pkg = open(path);
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        limits,
        &ImportLimits::default(),
    )
}

fn named<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
}

#[test]
fn the_structure_comes_from_relationships_in_a_fixed_order() {
    let s = slides(RICH);
    let st = &s.parts.structure;
    let kinds: Vec<(SlideKind, &str)> =
        st.parts.iter().map(|e| (e.kind, e.part.as_str())).collect();
    assert_eq!(
        kinds,
        [
            (SlideKind::Master, "ppt/slideMasters/slideMaster1.xml"),
            (SlideKind::Layout, "ppt/slideLayouts/slideLayout1.xml"),
            (SlideKind::Layout, "ppt/slideLayouts/slideLayout2.xml"),
            (SlideKind::Layout, "ppt/slideLayouts/slideLayout3.xml"),
            (SlideKind::Slide, "ppt/slides/slide1.xml"),
            (SlideKind::Slide, "ppt/slides/slide2.xml"),
            (SlideKind::Slide, "ppt/slides/slide3.xml"),
            (SlideKind::Notes, "ppt/notesSlides/notesSlide1.xml"),
            (SlideKind::NotesMaster, "ppt/notesMasters/notesMaster1.xml"),
            (
                SlideKind::HandoutMaster,
                "ppt/handoutMasters/handoutMaster1.xml"
            ),
        ]
    );
    assert_eq!(st.slides, [4, 5, 6]);
    assert_eq!(st.masters, [0]);
    assert_eq!(st.layouts, [1, 2, 3]);
    assert_eq!(st.notes, [7]);
    assert_eq!((st.notes_master, st.handout_master), (Some(8), Some(9)));
    let e = |o: u32| st.entry(o).unwrap();
    assert_eq!(
        (e(4).layout, e(5).layout, e(6).layout),
        (Some(1), Some(2), Some(3))
    );
    assert_eq!(e(6).notes, Some(7));
    assert_eq!(e(7).master, Some(8), "a notes slide's notes master");
    assert!([1, 2, 3].iter().all(|&l| e(l).master == Some(0)));
    assert_eq!(e(0).theme.as_deref(), Some("ppt/theme/theme1.xml"));
    let structural = s
        .findings
        .iter()
        .filter(|f| !matches!(f.kind, K::RetainedElement | K::RetainedAttribute))
        .collect::<Vec<_>>();
    assert!(structural.is_empty(), "{structural:?}");
}

#[test]
fn part_attributes_read_as_written_and_absent_stays_absent() {
    let s = slides(RICH);
    let p = |o| s.parts.get(o).unwrap();
    let master = p(0);
    assert_eq!(master.preserve, Some(true));
    let hf = master.header_footer.unwrap();
    assert_eq!(
        (hf.header, hf.footer, hf.date_time, hf.slide_number),
        (Some(false), Some(false), Some(false), None),
        "an attribute not written is not the schema's true"
    );
    let styles = master.text_styles.as_ref().unwrap();
    assert!(styles.title.is_some() && styles.body.is_some() && styles.other.is_some());
    let ids = master.layout_ids.as_ref().unwrap();
    assert_eq!(ids.len(), 3);
    assert_eq!(
        (ids[0].id, ids[0].rel_id.as_str()),
        (Some(2_147_483_649), "rId1")
    );
    assert_eq!(
        master.color_map.as_ref().unwrap().get(ColorMapSlot::Bg1),
        Some(ColorSchemeIndex::Lt1)
    );
    match &master
        .common
        .as_ref()
        .unwrap()
        .background
        .as_ref()
        .unwrap()
        .fill
    {
        Some(BackgroundFill::Reference(r)) => assert_eq!(r.idx, Some(1001)),
        other => panic!("{other:?}"),
    }
    let l1 = p(1);
    assert_eq!(l1.layout_type, Some(SlideLayoutType::Title));
    assert_eq!(l1.matching_name.as_deref(), Some("Title"));
    assert_eq!((l1.preserve, l1.user_drawn), (Some(true), Some(true)));
    assert_eq!(l1.show_master_shapes, None);
    assert_eq!(l1.color_map_override, Some(ColorMapOverride::Master));
    assert_eq!(l1.header_footer.unwrap().slide_number, Some(false));
    let l3 = p(3);
    assert_eq!(
        (l3.show_master_shapes, l3.show_master_placeholder_animations),
        (Some(false), Some(false))
    );
    let bg = l3.common.as_ref().unwrap().background.as_ref().unwrap();
    assert_eq!(bg.bw_mode, Some(BlackWhiteMode::Black));
    match &bg.fill {
        Some(BackgroundFill::Properties(pr)) => {
            assert_eq!(pr.shade_to_title, Some(true));
            assert!(pr.has_fill && pr.has_effects);
        }
        other => panic!("{other:?}"),
    }
    match &l3.color_map_override {
        Some(ColorMapOverride::Override(map)) => {
            assert_eq!(map.get(ColorMapSlot::Bg1), Some(ColorSchemeIndex::Dk1));
        }
        other => panic!("{other:?}"),
    }
    let s1 = p(4);
    assert_eq!((s1.show, s1.show_master_shapes), (None, None));
    assert_eq!(s1.common.as_ref().unwrap().name, None);
    let s3 = p(6);
    assert_eq!(
        (
            s3.show,
            s3.show_master_shapes,
            s3.show_master_placeholder_animations
        ),
        (Some(false), Some(false), Some(false))
    );
    assert_eq!(
        s3.common.as_ref().unwrap().name.as_deref(),
        Some("Hidden dark")
    );
    assert!(p(8).notes_style.is_some());
    assert_eq!(p(8).header_footer.unwrap().date_time, Some(false));
    assert_eq!(p(9).header_footer.unwrap().header, Some(false));
    assert_eq!(p(7).kind, SlideKind::Notes);
}

#[test]
fn every_node_kind_has_identity_transform_locks_and_links() {
    let s = slides(RICH);
    let slide = s.parts.get(5).unwrap();
    let tree = slide.shape_tree().unwrap();
    assert_eq!(tree.id, ObjectId { part: 5, node: 1 });
    let kinds: Vec<(u32, NodeKind)> = tree.descendants().map(|n| (n.id.node, n.kind)).collect();
    assert_eq!(
        kinds,
        [
            (1, NodeKind::Group),
            (2, NodeKind::Shape),
            (3, NodeKind::Shape),
            (4, NodeKind::Shape),
            (5, NodeKind::Shape),
            (6, NodeKind::Group),
            (7, NodeKind::Shape),
            (8, NodeKind::Shape),
            (9, NodeKind::Group),
            (10, NodeKind::Shape),
            (11, NodeKind::Connector),
            (12, NodeKind::Picture),
            (13, NodeKind::GraphicFrame),
            (14, NodeKind::Shape),
            (15, NodeKind::ContentPart),
        ],
        "ids in document order, the alternate content's selected branch in place"
    );
    let title = named(slide, "Title 1");
    let nv = title.non_visual.as_ref().unwrap();
    let click = nv.hyperlink_click.as_ref().unwrap();
    assert_eq!(click.rel_id.as_deref(), Some("rId4"));
    assert_eq!(
        Action::parse(click.action.as_deref().unwrap()),
        Action::SlideJump
    );
    assert_eq!(
        (click.history, click.tooltip.as_deref()),
        (Some(false), Some("To slide 3"))
    );
    let hover = nv.hyperlink_hover.as_ref().unwrap();
    assert_eq!(hover.rel_id.as_deref(), Some(""), "empty is not absent");
    assert_eq!(hover.end_sound, Some(true));
    assert_eq!(
        title.transform, None,
        "a placeholder with no xfrm has none here"
    );

    let content = named(slide, "Content Placeholder 2");
    let ph = content.placeholder().unwrap();
    assert_eq!(
        (ph.placeholder_type, ph.idx),
        (None, Some(1)),
        "type obj is not filled in"
    );
    let orphan = named(slide, "Orphan Placeholder").placeholder().unwrap();
    assert_eq!(orphan.idx, Some(u32::MAX));
    assert_eq!(
        named(slide, "Hidden").non_visual.as_ref().unwrap().hidden,
        Some(true)
    );

    let group = named(slide, "Group 9");
    let t = group.transform.unwrap();
    assert_eq!(
        (t.rotation, t.flip_h, t.flip_v),
        (Some(5_400_000), Some(true), None)
    );
    assert_eq!(
        t.child_extent,
        Some(PositiveSize {
            cx: 1_828_800,
            cy: 914_400
        })
    );
    assert_eq!(group.bw_mode, Some(BlackWhiteMode::Auto));
    let locks = group.locks.unwrap();
    assert_eq!(
        (locks.no_ungrp, locks.no_rot, locks.no_move),
        (Some(true), Some(true), None)
    );
    let a = named(slide, "Child A");
    assert_eq!(a.text_box, Some(true));
    assert_eq!(a.locks.unwrap().no_text_edit, Some(true));
    let b = named(slide, "Child B");
    assert_eq!(
        (b.use_bg_fill, b.bw_mode),
        (Some(true), Some(BlackWhiteMode::Gray))
    );
    assert_eq!(named(slide, "Inner Child").id.node, 10);

    let cxn = named(slide, "Connector 14");
    assert_eq!(cxn.start_connection, Some(Connection { spid: 11, site: 3 }));
    assert_eq!(cxn.end_connection, Some(Connection { spid: 12, site: 1 }));
    assert_eq!(cxn.locks.unwrap().no_change_shape_type, Some(true));
    assert_eq!(cxn.transform.unwrap().flip_v, Some(true));
    let pic = named(slide, "Picture 15");
    assert_eq!(pic.locks.unwrap().no_crop, Some(true));
    assert_eq!(
        pic.placeholder().unwrap().placeholder_type,
        Some(PlaceholderType::Pic)
    );
    assert_eq!(
        pic.placeholder().unwrap().size,
        Some(PlaceholderSize::Quarter)
    );
    let frame = named(slide, "Table 16");
    assert_eq!(
        frame.graphic_uri.as_deref(),
        Some("http://schemas.openxmlformats.org/drawingml/2006/table")
    );
    assert_eq!(frame.bw_mode, Some(BlackWhiteMode::Auto));
    assert_eq!(frame.locks.unwrap().no_drilldown, Some(true));
    assert_eq!(
        frame.transform.unwrap().offset,
        Some(Point {
            x: Coordinate(6_000_000),
            y: Coordinate(1_825_625)
        })
    );
    assert!(
        named(slide, "Fallback Shape").non_visual.is_some(),
        "p14 is not understood"
    );
    let part = tree.children.last().unwrap();
    assert_eq!(
        (part.kind, part.rel_id.as_deref()),
        (NodeKind::ContentPart, Some("rId5"))
    );

    let s1 = s.parts.get(4).unwrap();
    let nv = named(s1, "Title 1").non_visual.clone().unwrap();
    assert_eq!(
        nv.creation_id.as_deref(),
        Some("{6C3B0F0E-1D2A-4B5C-9D8E-0F1A2B3C4D5E}")
    );
    assert_eq!(nv.decorative, Some(false));
    let sub = named(s1, "Subtitle 2").non_visual.clone().unwrap();
    assert_eq!(
        (sub.descr.as_deref(), sub.title.as_deref()),
        (Some("Alt text"), Some("Subtitle"))
    );
    let l1 = s.parts.get(1).unwrap();
    let ct = named(l1, "Title 1").placeholder().unwrap();
    assert_eq!(
        (ct.placeholder_type, ct.orient, ct.has_custom_prompt),
        (
            Some(PlaceholderType::CtrTitle),
            Some(Direction::Horz),
            Some(true)
        )
    );
    // What a node holds beyond this tier is in the report, by name. A table no longer
    // is: Tier 1G reads it.
    assert!(
        !s.findings.has(K::RetainedElement, "tbl"),
        "{:?}",
        s.findings
    );
    assert!(s.findings.has(K::RetainedElement, "AlternateContent"));
    assert_eq!(
        s.parts.get(8).unwrap().shape_tree().unwrap().children.len(),
        3
    );
}

#[test]
fn ids_do_not_depend_on_spid_or_on_load_order() {
    let eager = slides(RICH);
    let mut pkg = open(RICH);
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let mut findings = Findings::new(100);
    let structure = index_deck(
        &pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &mut findings,
    )
    .unwrap();
    let mut lazy = LazySlides::new(structure, SlideLimits::default(), ImportLimits::default());
    let third = lazy.slide(&mut pkg, 2).unwrap().value.clone();
    let first = lazy.slide(&mut pkg, 0).unwrap().value.clone();
    assert_eq!(Some(&third), eager.parts.slide(2));
    assert_eq!(Some(&first), eager.parts.slide(0));
    // Two shapes share spid 2: ids stay distinct, and the duplicate is reported.
    let dup = slides("hostile/slide-duplicate-ids.bin");
    let tree = dup.parts.slide(0).unwrap().shape_tree().unwrap();
    let ids: Vec<u32> = tree.descendants().map(|n| n.id.node).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5]);
    assert!(dup.findings.has(K::DuplicateId, "cNvPr"));
}

#[test]
fn a_500_slide_deck_is_indexed_without_reading_a_slide() {
    let path = "generated/five-hundred-slides.pptx";
    let mut pkg = open(path);
    // Make every slide but the first unreadable. Indexing and loading slide 1 must
    // not notice, which they would if they parsed the others.
    let mut edits = PartEdits::default();
    for i in 2..=500 {
        edits.replace(format!("ppt/slides/slide{i}.xml"), b"<not xml".to_vec());
    }
    let broken = pkg.save_with(&edits).unwrap();
    let mut pkg = PresentationPackage::open(broken, OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let mut findings = Findings::new(100);
    let structure = index_deck(
        &pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &mut findings,
    )
    .unwrap();
    assert_eq!(structure.slides.len(), 500);
    let mut lazy = LazySlides::new(structure, SlideLimits::default(), ImportLimits::default());
    assert_eq!(lazy.loaded_count(), 0);
    let first = lazy.slide(&mut pkg, 0).unwrap();
    assert_eq!(first.value.kind, SlideKind::Slide);
    assert_eq!(lazy.loaded_count(), 1);
    let err = lazy.slide(&mut pkg, 499).unwrap_err();
    assert_eq!(
        err.code(),
        "CS-XML-0004",
        "a broken slide is found only when read"
    );
    let missing = lazy.part(&mut pkg, 99_999).unwrap_err();
    assert_eq!(missing.code(), "CS-SLD-0005");
}

#[test]
fn hostile_slides_refuse_with_their_code_or_report() {
    let limits = SlideLimits::default();
    let code = |path: &str| try_slides(path, &limits).map(|_| ()).unwrap_err().code();
    assert_eq!(code("hostile/slide-limit-shapes.bin"), "CS-SLD-0001");
    assert_eq!(code("hostile/slide-limit-group-depth.bin"), "CS-SLD-0002");
    let tight = SlideLimits {
        max_slides: 2,
        ..SlideLimits::default()
    };
    assert_eq!(
        try_slides("generated/three-slides.pptx", &tight)
            .unwrap_err()
            .code(),
        "CS-SLD-0003"
    );
    let parts = SlideLimits {
        max_parts: 3,
        ..SlideLimits::default()
    };
    assert_eq!(try_slides(RICH, &parts).unwrap_err().code(), "CS-SLD-0004");
    let ids = SlideLimits {
        max_layout_ids: 2,
        ..SlideLimits::default()
    };
    assert_eq!(try_slides(RICH, &ids).unwrap_err().code(), "CS-SLD-0004");
    // Just under each bound, the same decks read.
    let roomy = SlideLimits {
        max_shapes_per_part: 100_003,
        max_group_depth: 70,
        ..SlideLimits::default()
    };
    assert!(try_slides("hostile/slide-limit-shapes.bin", &roomy).is_ok());
    assert!(try_slides("hostile/slide-limit-group-depth.bin", &roomy).is_ok());

    // A cycle is a wrong-kind relationship, reported, and no chain.
    let cycle = slides("hostile/slide-layout-cycle.bin");
    let slide = cycle.parts.structure.slides[0];
    assert_eq!(cycle.parts.structure.entry(slide).unwrap().layout, None);
    assert!(cycle.findings.has(K::WrongRelationship, "slideLayout"));
    assert!(cycle.findings.has(K::WrongRelationship, "slideMaster"));
    // A real part of another kind, reached first through this relationship, is not a layout.
    let wrong = slides("hostile/slide-layout-wrong-kind.bin");
    let slide = wrong.parts.structure.slides[0];
    assert_eq!(wrong.parts.structure.entry(slide).unwrap().layout, None);
    assert!(wrong.findings.has(K::WrongRelationship, "slideLayout"));
    assert!(
        wrong
            .parts
            .structure
            .parts
            .iter()
            .all(|e| !e.part.contains("theme"))
    );
    for path in [
        "hostile/slide-no-layout.bin",
        "hostile/slide-missing-layout.bin",
    ] {
        let s = slides(path);
        let slide = s.parts.structure.slides[0];
        assert_eq!(
            s.parts.structure.entry(slide).unwrap().layout,
            None,
            "{path}"
        );
        assert!(
            s.parts.get(slide).is_some(),
            "{path}: the slide still reads"
        );
        let reported = s.findings.has(K::UnresolvedReference, "slideLayout")
            || s.findings.has(K::UnusableRelationship, "slideLayout");
        assert!(reported, "{path}: {:?}", s.findings);
    }
}

#[test]
fn bad_values_are_reported_and_left_out_never_coerced() {
    let s = slides("hostile/slide-bad-values.bin");
    let slide = s.parts.slide(0).unwrap();
    let tree = slide.shape_tree().unwrap();
    let sp = &tree.children[0];
    assert_eq!(sp.use_bg_fill, None);
    let nv = sp.non_visual.as_ref().unwrap();
    assert_eq!((nv.spid, nv.hidden), (None, None));
    let link = nv.hyperlink_click.as_ref().unwrap();
    assert!(Action::parse(link.action.as_deref().unwrap()).runs_code());
    assert_eq!(sp.text_box, None);
    let locks = sp.locks.unwrap();
    assert_eq!(
        (locks.no_move, locks.no_ungrp),
        (None, None),
        "noUngrp is not a shape lock"
    );
    let ph = sp.placeholder().unwrap();
    assert_eq!((ph.placeholder_type, ph.idx, ph.size), (None, None, None));
    let t = sp.transform.unwrap();
    assert_eq!(
        (t.rotation, t.flip_h, t.offset, t.extent),
        (None, None, None, None)
    );
    assert_eq!(sp.bw_mode, None);
    assert_eq!(tree.children[1].start_connection, None);
    assert_eq!(tree.children[2].graphic_uri, None);
    assert_eq!(tree.children[3].rel_id, None);
    assert_eq!(
        tree.children.len(),
        4,
        "an unknown node is retained, not a node"
    );
    for (kind, element, attr) in [
        (K::MalformedValue, "sp", Some("useBgFill")),
        (K::MalformedValue, "cNvPr", Some("id")),
        (K::DanglingRelationship, "hlinkClick", Some("r:id")),
        (K::MalformedValue, "ph", Some("type")),
        (K::MalformedValue, "off", Some("x")),
        (K::OutOfRange, "ext", Some("cx")),
        (K::RetainedAttribute, "spLocks", Some("noUngrp")),
        (K::MissingRequired, "stCxn", Some("idx")),
        (K::MissingRequired, "graphicData", Some("uri")),
        (K::MissingRequired, "contentPart", Some("r:id")),
        (K::RetainedElement, "unknownNode", None),
    ] {
        let found = s
            .findings
            .iter()
            .any(|f| f.kind == kind && f.element == element && f.attribute.as_deref() == attr);
        assert!(found, "{kind:?} {element} {attr:?}: {:?}", s.findings);
    }
}

/// BEH-21: nothing the file did not say is in the model, on any node or part.
#[test]
fn absent_stays_absent_on_a_bare_node_and_part() {
    let s = slides("generated/minimal.pptx");
    let slide = s.parts.slide(0).unwrap();
    assert_eq!(
        (
            slide.show,
            slide.show_master_shapes,
            slide.show_master_placeholder_animations
        ),
        (None, None, None)
    );
    assert_eq!(slide.color_map_override, None);
    assert_eq!(slide.common.as_ref().unwrap().background, None);
    let tree = slide.shape_tree().unwrap();
    assert_eq!(
        (tree.transform, tree.bw_mode, tree.locks),
        (None, None, None)
    );
    let title = &tree.children[0];
    let mut expected = ShapeNode::new(title.id, NodeKind::Shape);
    expected.non_visual = title.non_visual.clone();
    expected.app = title.app.clone();
    expected.transform = title.transform;
    expected.text_body = title.text_body.clone();
    expected.properties = title.properties.clone();
    assert_eq!(*title, expected, "no field the file did not state");
    assert!(title.properties.fill.is_none() && title.properties.line.is_none());
    let nv = title.non_visual.as_ref().unwrap();
    assert_eq!(
        (nv.descr.as_deref(), nv.hidden, nv.title.as_deref()),
        (None, None, None)
    );
    assert_eq!(
        (nv.hyperlink_click.as_ref(), nv.creation_id.as_deref()),
        (None, None)
    );
    let t = title.transform.unwrap();
    assert_eq!((t.rotation, t.flip_h, t.flip_v), (None, None, None));
    let ph = title.placeholder().unwrap();
    assert_eq!(
        (ph.idx, ph.size, ph.orient, ph.has_custom_prompt),
        (None, None, None, None)
    );
}
