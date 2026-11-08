use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::{CommonSlideData, SlideKind, SlidePart};

use super::*;

const SHAPE: ObjectId = ObjectId { part: 0, node: 2 };

fn run(text: &str, bold: Option<bool>) -> TextItem {
    TextItem::Run(TextRun {
        properties: bold.map(|b| {
            Box::new(CharacterProperties {
                bold: Some(b),
                ..Default::default()
            })
        }),
        text: Some(text.to_owned()),
    })
}

fn para(items: Vec<TextItem>) -> Paragraph {
    Paragraph {
        items,
        ..Default::default()
    }
}

fn deck(paragraphs: Vec<Paragraph>) -> SlideParts {
    let mut shape = ShapeNode::new(SHAPE, NodeKind::Shape);
    shape.text_body = Some(Box::new(TextBody {
        paragraphs,
        ..Default::default()
    }));
    let mut tree = ShapeNode::new(ObjectId { part: 0, node: 1 }, NodeKind::Group);
    tree.children.push(shape);
    let mut part = SlidePart::new(ObjectId::part(0), SlideKind::Slide);
    part.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(tree),
        controls: Vec::new(),
    });
    SlideParts {
        parts: vec![Some(part)],
        ..Default::default()
    }
}

fn body(parts: &SlideParts) -> &TextBody {
    parts
        .get(0)
        .unwrap()
        .node(SHAPE)
        .unwrap()
        .text_body
        .as_deref()
        .unwrap()
}

fn text(parts: &SlideParts) -> String {
    body(parts)
        .paragraphs
        .iter()
        .map(|p| {
            p.items
                .iter()
                .map(|i| match i {
                    TextItem::Run(r) => r.text.clone().unwrap_or_default(),
                    TextItem::Break(_) => "\u{b}".to_owned(),
                    _ => String::new(),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn sample() -> SlideParts {
    deck(vec![
        para(vec![run("Hello ", None), run("world", Some(true))]),
        para(vec![run("Second", None)]),
    ])
}

/// Apply, check, then apply the inverse and require the very model back.
fn round_trip(op: Operation, expect: &str) -> SlideParts {
    let mut parts = sample();
    let original = parts.clone();
    let inverse = apply(&mut parts, op).unwrap();
    assert_eq!(text(&parts), expect);
    let edited = parts.clone();
    let redo = apply(&mut parts, inverse).unwrap();
    assert_eq!(parts, original, "the inverse gives back the model as read");
    apply(&mut parts, redo).unwrap();
    assert_eq!(parts, edited, "the inverse of the inverse redoes it");
    edited
}

#[test]
fn typed_text_joins_the_run_before_it_and_takes_its_properties() {
    let edited = round_trip(
        Operation::InsertText {
            object: SHAPE,
            at: 11,
            text: "!".into(),
        },
        "Hello world!\nSecond",
    );
    let first = &body(&edited).paragraphs[0];
    assert_eq!(first.items.len(), 2, "no new run");
    assert_eq!(first.items[1], run("world!", Some(true)));
}

#[test]
fn a_line_feed_splits_the_paragraph_and_a_vertical_tab_is_a_line_break() {
    round_trip(
        Operation::InsertText {
            object: SHAPE,
            at: 5,
            text: ",\nnew".into(),
        },
        "Hello,\nnew world\nSecond",
    );
    round_trip(
        Operation::InsertText {
            object: SHAPE,
            at: 0,
            text: "a\u{b}b".into(),
        },
        "a\u{b}bHello world\nSecond",
    );
}

#[test]
fn control_characters_xml_cannot_hold_are_dropped() {
    round_trip(
        Operation::InsertText {
            object: SHAPE,
            at: 0,
            text: "x\u{0}\u{1b}\ty".into(),
        },
        "x\tyHello world\nSecond",
    );
}

#[test]
fn a_delete_within_and_across_runs_and_paragraphs_inverts_exactly() {
    round_trip(
        Operation::DeleteRange {
            object: SHAPE,
            start: 3,
            end: 8,
        },
        "Helrld\nSecond",
    );
    let merged = round_trip(
        Operation::DeleteRange {
            object: SHAPE,
            start: 9,
            end: 15,
        },
        "Hello worond",
    );
    assert_eq!(body(&merged).paragraphs.len(), 1);
}

#[test]
fn formatting_a_range_splits_runs_at_its_ends() {
    let edited = round_trip(
        Operation::FormatRange {
            object: SHAPE,
            start: 2,
            end: 8,
            attribute: Attribute::Italic(Some(true)),
        },
        "Hello world\nSecond",
    );
    let items = &body(&edited).paragraphs[0].items;
    assert_eq!(items.len(), 4);
    let italic = |i: &TextItem| properties_of(i).and_then(|p| p.italic) == Some(true);
    assert_eq!(
        items.iter().map(italic).collect::<Vec<_>>(),
        [false, true, true, false]
    );
}

#[test]
fn an_offset_past_the_end_is_refused_and_changes_nothing() {
    let mut parts = sample();
    let before = parts.clone();
    let err = apply(
        &mut parts,
        Operation::DeleteRange {
            object: SHAPE,
            start: 0,
            end: 99,
        },
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0004");
    assert_eq!(parts, before);
    let missing = ObjectId { part: 0, node: 9 };
    let err = apply(
        &mut parts,
        Operation::InsertText {
            object: missing,
            at: 0,
            text: "x".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0001");
}

#[test]
fn a_failing_batch_takes_back_what_it_did() {
    let mut parts = sample();
    let before = parts.clone();
    let err = apply(
        &mut parts,
        Operation::Batch(vec![
            Operation::InsertText {
                object: SHAPE,
                at: 0,
                text: "x".into(),
            },
            Operation::DeleteRange {
                object: SHAPE,
                start: 0,
                end: 999,
            },
        ]),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0004");
    assert_eq!(parts, before);
}

#[test]
fn history_groups_typing_into_one_undo_and_redoes_it() {
    let mut parts = sample();
    let original = parts.clone();
    let mut history = History::new();
    let mark = history.undo_mark();
    for (n, ch) in "abc".chars().enumerate() {
        history
            .apply(
                &mut parts,
                Operation::InsertText {
                    object: SHAPE,
                    at: n,
                    text: ch.to_string(),
                },
            )
            .unwrap();
        assert!(history.group_since(mark));
    }
    assert_eq!(text(&parts), "abcHello world\nSecond");
    history.undo(&mut parts).unwrap().unwrap();
    assert_eq!(parts, original);
    assert!(!history.can_undo());
    history.redo(&mut parts).unwrap().unwrap();
    assert_eq!(text(&parts), "abcHello world\nSecond");
    assert_eq!(history.edits_applied(), 5);
}

#[test]
fn an_edit_that_changes_nothing_leaves_no_undo_and_keeps_redo() {
    let mut parts = sample();
    let mut history = History::new();
    history
        .apply(
            &mut parts,
            Operation::InsertText {
                object: SHAPE,
                at: 0,
                text: "x".into(),
            },
        )
        .unwrap();
    history.undo(&mut parts).unwrap();
    let changed = history
        .apply(
            &mut parts,
            Operation::DeleteRange {
                object: SHAPE,
                start: 2,
                end: 2,
            },
        )
        .unwrap();
    assert!(!changed);
    assert!(history.can_redo());
    assert!(!history.can_undo());
}

// `UX-003`: the Home tab's Font and Paragraph groups.

fn props_of(parts: &SlideParts, p: usize, i: usize) -> Option<CharacterProperties> {
    match body(parts).paragraphs[p].items.get(i)? {
        TextItem::Run(r) => r.properties.as_deref().cloned(),
        _ => None,
    }
}

#[test]
fn size_font_colour_and_highlight_format_only_their_range_and_undo_exactly() {
    use casual_slide_model::color::Color;
    use casual_slide_model::fill::Fill;
    let font = TextFont {
        typeface: "Georgia".to_owned(),
        panose: None,
        pitch_family: None,
        charset: None,
    };
    let red = Color::srgb(0xFF, 0, 0);
    for attribute in [
        Attribute::Size(Some(3200)),
        Attribute::Latin(Some(Box::new(font.clone()))),
        Attribute::Fill(Some(Fill::Solid(Some(red.clone())))),
        Attribute::Highlight(Some(red.clone())),
    ] {
        let mut parts = sample();
        let before = parts.clone();
        let mut history = History::new();
        // "Hello |wor|ld": the run splits and only the middle takes the attribute.
        let changed = history
            .apply(
                &mut parts,
                Operation::FormatRange {
                    object: SHAPE,
                    start: 6,
                    end: 9,
                    attribute: attribute.clone(),
                },
            )
            .unwrap();
        assert!(changed, "{attribute:?}");
        assert_eq!(text(&parts), "Hello world\nSecond");
        let mid = props_of(&parts, 0, 1).unwrap();
        let mut want = CharacterProperties {
            bold: Some(true),
            ..Default::default()
        };
        attribute.set(&mut want);
        assert_eq!(mid, want, "{attribute:?}");
        // Outside the range nothing changed.
        assert_eq!(props_of(&parts, 0, 0), None);
        assert_eq!(props_of(&parts, 0, 2).unwrap().size, None);
        history.undo(&mut parts).unwrap();
        assert_eq!(parts, before, "{attribute:?}");
    }
}

#[test]
fn clearing_an_attribute_removes_it_and_highlight_none_removes_the_element() {
    let mut parts = sample();
    let mut history = History::new();
    let fmt = |a| Operation::FormatRange {
        object: SHAPE,
        start: 0,
        end: 5,
        attribute: a,
    };
    history
        .apply(&mut parts, fmt(Attribute::Size(Some(2400))))
        .unwrap();
    history
        .apply(&mut parts, fmt(Attribute::Highlight(None)))
        .unwrap();
    let p = props_of(&parts, 0, 0).unwrap();
    assert_eq!((p.size, p.highlight), (Some(2400), None));
    history
        .apply(&mut parts, fmt(Attribute::Size(None)))
        .unwrap();
    assert_eq!(props_of(&parts, 0, 0).unwrap().size, None);
}

#[test]
fn paragraph_alignment_and_bullets_set_only_the_paragraphs_named_and_undo() {
    let mut parts = sample();
    let before = parts.clone();
    let mut history = History::new();
    let change = |c| Operation::FormatParagraphs {
        object: SHAPE,
        first: 1,
        count: 1,
        change: c,
    };
    assert!(
        history
            .apply(
                &mut parts,
                change(ParagraphChange::Align(Some(TextAlign::Center)))
            )
            .unwrap()
    );
    assert!(
        history
            .apply(
                &mut parts,
                change(ParagraphChange::Bullet(
                    Some(Bullet::Char(Some("•".to_owned()))),
                    None
                ))
            )
            .unwrap()
    );
    assert!(
        history
            .apply(
                &mut parts,
                change(ParagraphChange::Indent {
                    margin_left: Some(285_750),
                    indent: Some(-285_750),
                })
            )
            .unwrap()
    );
    let p1 = body(&parts).paragraphs[1].properties.as_deref().unwrap();
    assert_eq!(p1.align, Some(TextAlign::Center));
    assert_eq!(p1.bullet, Some(Bullet::Char(Some("•".to_owned()))));
    assert_eq!((p1.margin_left, p1.indent), (Some(285_750), Some(-285_750)));
    assert!(body(&parts).paragraphs[0].properties.is_none());
    assert_eq!(text(&parts), "Hello world\nSecond");
    for _ in 0..3 {
        history.undo(&mut parts).unwrap();
    }
    assert_eq!(parts, before);
}

#[test]
fn clearing_an_absent_paragraph_property_changes_nothing_and_writes_no_ppr() {
    let mut parts = sample();
    let before = parts.clone();
    let mut history = History::new();
    let changed = history
        .apply(
            &mut parts,
            Operation::FormatParagraphs {
                object: SHAPE,
                first: 0,
                count: 2,
                change: ParagraphChange::Align(None),
            },
        )
        .unwrap();
    assert!(!changed);
    assert_eq!(parts, before);
    assert!(!history.can_undo());
}

#[test]
fn a_paragraph_range_past_the_body_is_refused_before_anything_changes() {
    let mut parts = sample();
    let before = parts.clone();
    for (first, count) in [(1, 2), (2, 1), (0, 0), (usize::MAX, 2)] {
        let e = apply(
            &mut parts,
            Operation::FormatParagraphs {
                object: SHAPE,
                first,
                count,
                change: ParagraphChange::Align(Some(TextAlign::Right)),
            },
        )
        .unwrap_err();
        assert!(matches!(e, TxnError::OutOfRange { .. }));
        assert_eq!(parts, before);
    }
}
