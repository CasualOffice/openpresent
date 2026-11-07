//! BEH-01, BEH-02 and BEH-20 as tables: placeholder matching, inheritance order per
//! property family, and master-shape and background visibility.

use std::path::Path;

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ObjectId, Placeholder, PlaceholderType as T, ShapeNode};
use casual_slide_model::slide::{CommonSlideData, SlideKind, SlidePart, SlideParts};
use casual_slide_model::theme::ColorMapSlot;
use casual_slide_model::theme::ColorSchemeIndex;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_style::{
    Chain, Level, TextStyle, match_by_exact_type, match_by_index, match_by_type,
};

fn rich() -> SlideParts {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/slides-masters-layouts.pptx");
    let mut pkg =
        PresentationPackage::open(std::fs::read(path).unwrap(), OoxmlLimits::default()).unwrap();
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
    .parts
}

fn named<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
}

fn name_of(n: &ShapeNode) -> &str {
    n.non_visual
        .as_ref()
        .and_then(|v| v.name.as_deref())
        .unwrap_or("")
}

/// A part holding placeholders: `(name, kind, type, idx)`.
fn part_with(
    kind: SlideKind,
    ordinal: u32,
    items: &[(&str, NodeKind, Option<T>, Option<u32>)],
) -> SlidePart {
    let mut root = ShapeNode::new(
        ObjectId {
            part: ordinal,
            node: 1,
        },
        NodeKind::Group,
    );
    for (i, (name, node_kind, t, idx)) in items.iter().enumerate() {
        let mut n = ShapeNode::new(
            ObjectId {
                part: ordinal,
                node: u32::try_from(i).unwrap() + 2,
            },
            *node_kind,
        );
        n.non_visual = Some(casual_slide_model::shape::NonVisualDrawing {
            name: Some((*name).to_owned()),
            ..Default::default()
        });
        n.app = Some(casual_slide_model::shape::AppNonVisual {
            placeholder: Some(Placeholder {
                placeholder_type: *t,
                idx: *idx,
                ..Placeholder::default()
            }),
            ..Default::default()
        });
        root.children.push(n);
    }
    let mut p = SlidePart::new(ObjectId::part(ordinal), kind);
    p.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
    });
    p
}

/// A slide placeholder case: what it is, its type, idx and kind, the layout match.
type SlideCase<'a> = (&'a str, Option<T>, Option<u32>, NodeKind, Option<&'a str>);

/// BEH-01: slide to layout by `idx` (MS-OI29500, Part 1 Section 19.3.1.36), layout to master by
/// type then class (Inferred), notes slide to notes master by type (documented).
#[test]
fn beh_01_placeholder_matching_table() {
    use NodeKind::{Connector, Shape};
    let layout = part_with(
        SlideKind::Layout,
        1,
        &[
            ("L title", Shape, Some(T::Title), None),
            ("L obj 1", Shape, None, Some(1)),
            ("L body 2", Shape, Some(T::Body), Some(2)),
            ("L max", Shape, Some(T::Body), Some(u32::MAX)),
            ("L dt 10", Shape, Some(T::Dt), Some(10)),
        ],
    );
    // (case, slide placeholder type, idx, node kind, expected layout match)
    let slide_cases: &[SlideCase<'_>] = &[
        (
            "absent idx is 0, matching the title",
            Some(T::Title),
            None,
            Shape,
            Some("L title"),
        ),
        (
            "idx alone matches, type left obj",
            None,
            Some(1),
            Shape,
            Some("L obj 1"),
        ),
        (
            "idx wins over a differing type",
            Some(T::Body),
            Some(1),
            Shape,
            Some("L obj 1"),
        ),
        (
            "same type, other idx: no match",
            Some(T::Body),
            Some(3),
            Shape,
            None,
        ),
        (
            "a date by idx",
            Some(T::Dt),
            Some(10),
            Shape,
            Some("L dt 10"),
        ),
        (
            "idx 4294967295 matches nothing",
            Some(T::Body),
            Some(u32::MAX),
            Shape,
            None,
        ),
        (
            "a connector is never a placeholder",
            None,
            Some(1),
            Connector,
            None,
        ),
        (
            "a title by type with another idx: no match",
            Some(T::Title),
            Some(5),
            Shape,
            None,
        ),
    ];
    for (case, t, idx, kind, expected) in slide_cases {
        let slide = part_with(SlideKind::Slide, 2, &[("S", *kind, *t, *idx)]);
        let node = &slide.shape_tree().unwrap().children[0];
        let chain = Chain {
            slide: Some(&slide),
            layout: Some(&layout),
            master: None,
        };
        let got = chain.inherited(node).layout.map(name_of);
        assert_eq!(got, *expected, "{case}");
        let direct = node
            .placeholder()
            .filter(|_| *kind == Shape)
            .and_then(|ph| match_by_index(ph, &layout))
            .map(name_of);
        assert_eq!(direct, *expected, "{case}, directly");
    }

    let master = part_with(
        SlideKind::Master,
        0,
        &[
            ("M title", Shape, Some(T::Title), None),
            ("M body", Shape, Some(T::Body), Some(1)),
            ("M dt", Shape, Some(T::Dt), Some(2)),
            ("M ftr", Shape, Some(T::Ftr), Some(3)),
            ("M sldNum", Shape, Some(T::SldNum), Some(4)),
        ],
    );
    let layout_cases: &[(Option<T>, Option<u32>, Option<&str>)] = &[
        (Some(T::Title), None, Some("M title")),
        (Some(T::CtrTitle), None, Some("M title")),
        (Some(T::SubTitle), Some(1), Some("M body")),
        (None, Some(1), Some("M body")),
        (Some(T::Pic), Some(13), Some("M body")),
        (Some(T::Tbl), Some(14), Some("M body")),
        (Some(T::Dt), Some(10), Some("M dt")),
        (Some(T::Ftr), Some(11), Some("M ftr")),
        (Some(T::SldNum), Some(12), Some("M sldNum")),
        (Some(T::Hdr), None, None),
        (Some(T::SldImg), None, None),
    ];
    for (t, idx, expected) in layout_cases {
        let ph = Placeholder {
            placeholder_type: *t,
            idx: *idx,
            ..Placeholder::default()
        };
        assert_eq!(match_by_type(&ph, &master).map(name_of), *expected, "{t:?}");
    }
    // A slide's master counterpart is its layout counterpart's.
    let slide = part_with(SlideKind::Slide, 2, &[("S", Shape, None, Some(2))]);
    let chain = Chain {
        slide: Some(&slide),
        layout: Some(&layout),
        master: Some(&master),
    };
    let found = chain.inherited(&slide.shape_tree().unwrap().children[0]);
    assert_eq!(
        (found.layout.map(name_of), found.master.map(name_of)),
        (Some("L body 2"), Some("M body"))
    );

    // Notes: by exact type, never by idx.
    let notes_master = part_with(
        SlideKind::NotesMaster,
        3,
        &[
            ("NM sldImg", Shape, Some(T::SldImg), Some(2)),
            ("NM body", Shape, Some(T::Body), Some(3)),
        ],
    );
    let notes = part_with(
        SlideKind::Notes,
        4,
        &[
            ("N body", Shape, Some(T::Body), Some(1)),
            ("N img", Shape, Some(T::SldImg), None),
            ("N obj", Shape, None, Some(3)),
        ],
    );
    let chain = Chain {
        slide: Some(&notes),
        layout: None,
        master: Some(&notes_master),
    };
    let got: Vec<Option<&str>> = notes
        .shape_tree()
        .unwrap()
        .children
        .iter()
        .map(|n| chain.inherited(n).master.map(name_of))
        .collect();
    assert_eq!(got, [Some("NM body"), Some("NM sldImg"), None]);
    let body = Placeholder {
        placeholder_type: Some(T::Body),
        ..Placeholder::default()
    };
    assert_eq!(
        match_by_exact_type(&body, &notes_master).map(name_of),
        Some("NM body")
    );
}

fn at(x: i64, y: i64) -> Point {
    Point {
        x: Coordinate(x),
        y: Coordinate(y),
    }
}

/// BEH-02: each family, on the deck, nearest level first.
#[test]
fn beh_02_inheritance_order_table() {
    let parts = rich();
    let (slide1, slide2, slide3) = (4, 5, 6);
    let chain = |o| Chain::of(&parts, o);
    let part = |o| parts.get(o).unwrap();

    // Transform: (slide, shape, level, offset).
    let transforms: &[(u32, &str, Level, Point)] = &[
        (slide1, "Title 1", Level::Layout, at(1_524_000, 1_122_363)),
        (slide1, "Subtitle 2", Level::Slide, at(1_524_000, 3_900_000)),
        (
            slide1,
            "Footer Placeholder 3",
            Level::Master,
            at(4_038_600, 6_356_350),
        ),
        (slide2, "Title 1", Level::Master, at(838_200, 365_125)),
        (
            slide2,
            "Content Placeholder 2",
            Level::Master,
            at(838_200, 1_825_625),
        ),
        (
            slide2,
            "Orphan Placeholder",
            Level::Slide,
            at(838_200, 5_500_000),
        ),
        (slide2, "Picture 15", Level::Layout, at(838_200, 4_000_000)),
    ];
    for (slide, name, level, offset) in transforms {
        let c = chain(*slide);
        let t = c.transform(named(part(*slide), name)).unwrap();
        assert_eq!(
            (t.from, t.value.offset),
            (*level, *offset),
            "{slide} {name}"
        );
    }
    let t = chain(slide2)
        .transform(named(part(slide2), "Group 9"))
        .unwrap();
    assert_eq!(
        (t.value.rotation, t.value.flip_h, t.value.flip_v),
        (5_400_000, true, false),
        "absent flipV is the schema default, read at resolve time"
    );
    assert_eq!(
        t.value.extent,
        PositiveSize {
            cx: 1_828_800,
            cy: 914_400
        }
    );

    // Placeholder type: the nearest that states one.
    let pt = chain(slide2)
        .placeholder_type(named(part(slide2), "Content Placeholder 2"))
        .unwrap();
    assert_eq!((pt.value, pt.from), (T::Body, Level::Master));
    let pt = chain(slide1)
        .placeholder_type(named(part(slide1), "Title 1"))
        .unwrap();
    assert_eq!((pt.value, pt.from), (T::CtrTitle, Level::Slide));

    // Colour map: (chain, level, what bg1 maps to).
    let maps: &[(Chain<'_>, Level, ColorSchemeIndex)] = &[
        (chain(slide1), Level::Master, ColorSchemeIndex::Lt1),
        (chain(slide2), Level::Master, ColorSchemeIndex::Lt1),
        (chain(slide3), Level::Slide, ColorSchemeIndex::Dk1),
        (chain(3), Level::Layout, ColorSchemeIndex::Dk1),
        (chain(0), Level::Master, ColorSchemeIndex::Lt1),
    ];
    for (c, level, bg1) in maps {
        let m = c.color_map().unwrap();
        assert_eq!(
            (m.from, m.value.get(ColorMapSlot::Bg1)),
            (*level, Some(*bg1))
        );
    }

    // Header and footer flags: (chain, level, [sldNum, hdr, ftr, dt]).
    let flags: &[(Chain<'_>, Level, [bool; 4])] = &[
        (chain(slide1), Level::Layout, [false, true, true, true]),
        (chain(slide2), Level::Master, [true, false, false, false]),
        (chain(slide3), Level::Layout, [true, true, true, true]),
    ];
    for (c, level, want) in flags {
        let hf = c.header_footer().unwrap();
        let got = [
            hf.value.slide_number,
            hf.value.header,
            hf.value.footer,
            hf.value.date_time,
        ];
        assert_eq!((hf.from, got), (*level, *want));
    }

    // Text style: (chain, slide, shape, style, level).
    let notes = 7;
    let styles: &[(u32, &str, TextStyle, Level)] = &[
        (slide1, "Title 1", TextStyle::Title, Level::Master),
        (slide1, "Subtitle 2", TextStyle::Body, Level::Master),
        (
            slide1,
            "Footer Placeholder 3",
            TextStyle::Other,
            Level::Master,
        ),
        (
            slide2,
            "Content Placeholder 2",
            TextStyle::Body,
            Level::Master,
        ),
        (slide2, "Hidden", TextStyle::Other, Level::Master),
        (
            notes,
            "Notes Placeholder 2",
            TextStyle::Notes,
            Level::Master,
        ),
        (
            notes,
            "Slide Image Placeholder 1",
            TextStyle::PresentationDefault,
            Level::Presentation,
        ),
    ];
    for (o, name, style, level) in styles {
        let s = chain(*o).text_style(named(part(*o), name));
        assert_eq!((s.value, s.from), (*style, *level), "{o} {name}");
    }
}

/// BEH-20: hidden slides, background inheritance, `showMasterSp`, which master and
/// layout shapes are drawn.
#[test]
fn beh_20_visibility_and_background_table() {
    let parts = rich();
    let part = |o| parts.get(o).unwrap();
    let (master, layout1, layout3, slide1, slide3) = (0, 1, 3, 4, 6);
    let mixed = Chain {
        slide: Some(part(slide1)),
        layout: Some(part(layout3)),
        master: Some(part(master)),
    };
    // (case, chain, hidden, layout shapes, master shapes, drawn, background level)
    let cases: &[(&str, Chain<'_>, Resolved3, &[&str], Level)] = &[
        (
            "nothing said",
            Chain::of(&parts, slide1),
            (
                (false, Level::SchemaDefault),
                (true, Level::SchemaDefault),
                (true, Level::SchemaDefault),
            ),
            &["Master Logo"],
            Level::Master,
        ),
        (
            "the slide hides master and layout shapes",
            Chain::of(&parts, slide3),
            (
                (true, Level::Slide),
                (false, Level::Slide),
                (false, Level::Slide),
            ),
            &[],
            Level::Slide,
        ),
        (
            "the layout hides the master's",
            mixed,
            (
                (false, Level::SchemaDefault),
                (true, Level::SchemaDefault),
                (false, Level::Layout),
            ),
            &["Layout Deco"],
            Level::Layout,
        ),
        (
            "a layout alone",
            Chain::of(&parts, layout1),
            (
                (false, Level::SchemaDefault),
                (true, Level::SchemaDefault),
                (true, Level::SchemaDefault),
            ),
            &["Master Logo"],
            Level::Master,
        ),
    ];
    for (case, c, (hidden, layout_vis, master_vis), drawn, bg) in cases {
        let h = c.hidden();
        assert_eq!((h.value, h.from), *hidden, "{case}: hidden");
        let l = c.layout_shapes_visible();
        assert_eq!((l.value, l.from), *layout_vis, "{case}: layout shapes");
        let m = c.master_shapes_visible();
        assert_eq!((m.value, m.from), *master_vis, "{case}: master shapes");
        let names: Vec<&str> = c.inherited_shapes().into_iter().map(name_of).collect();
        assert_eq!(names, *drawn, "{case}: drawn");
        assert_eq!(c.background().unwrap().from, *bg, "{case}: background");
    }
    let anim = Chain::of(&parts, slide3).master_placeholder_animations();
    assert_eq!((anim.value, anim.from), (false, Level::Slide));
}

type Resolved3 = ((bool, Level), (bool, Level), (bool, Level));

/// The resolver never writes: the model it read from is unchanged after every query.
#[test]
fn resolving_changes_nothing() {
    let parts = rich();
    let before = parts.clone();
    for o in 0..u32::try_from(parts.parts.len()).unwrap() {
        let c = Chain::of(&parts, o);
        for n in parts
            .get(o)
            .and_then(SlidePart::shape_tree)
            .into_iter()
            .flat_map(ShapeNode::descendants)
        {
            let _ = (
                c.transform(n),
                c.text_style(n),
                c.inherited(n),
                c.placeholder_type(n),
            );
        }
        let _ = (
            c.color_map(),
            c.background(),
            c.header_footer(),
            c.inherited_shapes(),
        );
    }
    assert_eq!(parts, before);
}
