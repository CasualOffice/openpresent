// Slice 12's tests (`VS-12`): design operations and their inverses.

use std::collections::BTreeMap;

use casual_slide_model::color::Color;
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::slide::{
    Background, BackgroundFill, BackgroundProperties, CommonSlideData, SlideKind, SlidePart,
    SlideParts,
};
use casual_slide_model::theme::{
    BaseStyles, ColorScheme, FontCollection, FontScheme, TextFont, Theme,
};

use crate::{History, Operation, apply};

use super::Document;

#[derive(Debug, Clone, PartialEq)]
struct Doc {
    parts: SlideParts,
    themes: BTreeMap<u32, Theme>,
    size: (i64, i64),
}

impl Document for Doc {
    fn parts_mut(&mut self) -> &mut SlideParts {
        &mut self.parts
    }
    fn theme_mut(&mut self, master: u32) -> Option<&mut Theme> {
        self.themes.get_mut(&master)
    }
    fn slide_size_mut(&mut self) -> Option<&mut (i64, i64)> {
        Some(&mut self.size)
    }
}

fn doc() -> Doc {
    let mut part = SlidePart::new(ObjectId::part(0), SlideKind::Master);
    part.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(ShapeNode::new(
            ObjectId { part: 0, node: 1 },
            NodeKind::Group,
        )),
        controls: Vec::new(),
    });
    let mut colors = ColorScheme {
        name: Some("Office".into()),
        ..ColorScheme::default()
    };
    colors.slots[0] = Some(Color::srgb(0, 0, 0));
    let theme = Theme {
        elements: Some(BaseStyles {
            color_scheme: Some(colors),
            font_scheme: Some(FontScheme::default()),
            ..BaseStyles::default()
        }),
        ..Theme::default()
    };
    Doc {
        parts: SlideParts {
            parts: vec![Some(part)],
            ..SlideParts::default()
        },
        themes: BTreeMap::from([(0, theme)]),
        size: (12_192_000, 6_858_000),
    }
}

fn solid(r: u8) -> Background {
    Background {
        bw_mode: None,
        fill: Some(BackgroundFill::Properties(BackgroundProperties {
            shade_to_title: None,
            has_fill: true,
            fill: Some(Fill::Solid(Some(Color::srgb(r, 0, 0)))),
            has_effects: true,
        })),
    }
}

fn ops() -> Vec<Operation> {
    let mut colors = ColorScheme {
        name: Some("Harbour".into()),
        ..ColorScheme::default()
    };
    colors.slots[4] = Some(Color::srgb(1, 2, 3));
    let fonts = FontScheme {
        name: Some("Harbour".into()),
        major: Some(FontCollection {
            latin: Some(TextFont {
                typeface: Some("Carlito".into()),
                ..TextFont::default()
            }),
            ..FontCollection::default()
        }),
        ..FontScheme::default()
    };
    vec![
        Operation::SetColorScheme {
            master: 0,
            scheme: Some(Box::new(colors)),
        },
        Operation::SetFontScheme {
            master: 0,
            scheme: Some(Box::new(fonts)),
        },
        Operation::SetSlideSize {
            cx: 9_144_000,
            cy: 6_858_000,
        },
        Operation::SetBackground {
            part: 0,
            background: Some(Box::new(solid(200))),
        },
    ]
}

#[test]
fn each_design_op_applied_then_its_inverse_gives_the_deck_back() {
    let mut all = ops();
    all.push(Operation::Batch(ops()));
    for op in all {
        let start = doc();
        let mut d = start.clone();
        let inverse = apply(&mut d, op.clone()).unwrap();
        assert_ne!(d, start, "{op:?} changed nothing");
        let again = apply(&mut d, inverse).unwrap();
        assert_eq!(d, start, "{op:?} not undone");
        let mut redone = start.clone();
        apply(&mut redone, again).unwrap();
        let mut direct = start.clone();
        apply(&mut direct, op.clone()).unwrap();
        assert_eq!(redone, direct, "{op:?} redo differs");
    }
}

#[test]
fn undo_and_redo_of_a_design_batch_is_one_step() {
    let start = doc();
    let mut d = start.clone();
    let mut h = History::new();
    assert!(h.apply(&mut d, Operation::Batch(ops())).unwrap());
    let after = d.clone();
    let undone = h.undo(&mut d).unwrap().unwrap();
    assert!(undone.is_deck_wide());
    assert_eq!(d, start);
    assert!(!h.can_undo());
    h.redo(&mut d).unwrap();
    assert_eq!(d, after);
}

#[test]
fn a_slide_size_outside_the_schema_is_refused_and_changes_nothing() {
    let start = doc();
    let mut d = start.clone();
    for (cx, cy) in [(914_399, 6_858_000), (9_144_000, 51_206_401), (-1, 0)] {
        let err = apply(&mut d, Operation::SetSlideSize { cx, cy }).unwrap_err();
        assert_eq!(err.code(), "CS-TXN-0007");
    }
    assert_eq!(d, start);
}

#[test]
fn a_design_op_on_parts_alone_or_an_unread_theme_is_refused() {
    let mut parts = doc().parts;
    let err = apply(
        &mut parts,
        Operation::SetSlideSize {
            cx: 9_144_000,
            cy: 9_144_000,
        },
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0006");
    let mut d = doc();
    let err = apply(
        &mut d,
        Operation::SetColorScheme {
            master: 7,
            scheme: None,
        },
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0006");
    // A failing member takes back the batch's earlier ones.
    let start = doc();
    let mut d = start.clone();
    let err = apply(
        &mut d,
        Operation::Batch(vec![
            Operation::SetBackground {
                part: 0,
                background: Some(Box::new(solid(9))),
            },
            Operation::SetBackground {
                part: 5,
                background: None,
            },
        ]),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0001");
    assert_eq!(d, start);
}

#[test]
fn setting_what_is_already_there_leaves_no_undo_entry() {
    let mut d = doc();
    let mut h = History::new();
    let (cx, cy) = d.size;
    assert!(!h.apply(&mut d, Operation::SetSlideSize { cx, cy }).unwrap());
    assert!(!h.can_undo());
}
