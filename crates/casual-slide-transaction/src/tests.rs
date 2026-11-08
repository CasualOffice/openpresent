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
