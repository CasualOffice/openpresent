// The Deck family's tests (`VS-08`): each op and its inverse, and what is refused.

use super::*;
use casual_slide_model::slide::{DeckStructure, PartEntry, SlideKind, SlidePart};

fn entry(ordinal: u32, kind: SlideKind) -> PartEntry {
    PartEntry {
        id: ObjectId::part(ordinal),
        kind,
        part: format!("ppt/slides/slide{ordinal}.xml"),
        layout: Some(0),
        master: None,
        notes: None,
        theme: None,
    }
}

/// A layout at ordinal 0 and three slides at 1, 2 and 3.
fn deck() -> SlideParts {
    let mut structure = DeckStructure {
        parts: vec![entry(0, SlideKind::Layout)],
        layouts: vec![0],
        ..DeckStructure::default()
    };
    let mut parts = vec![Some(SlidePart::new(ObjectId::part(0), SlideKind::Layout))];
    for n in 1..4 {
        structure.parts.push(entry(n, SlideKind::Slide));
        structure.slides.push(n);
        parts.push(Some(SlidePart::new(ObjectId::part(n), SlideKind::Slide)));
    }
    SlideParts { structure, parts }
}

fn new_slide(parts: &SlideParts, index: usize) -> Operation {
    let ordinal = u32::try_from(parts.parts.len()).unwrap();
    Operation::Batch(vec![
        Operation::AddPart {
            entry: Box::new(entry(ordinal, SlideKind::Slide)),
            part: Box::new(SlidePart::new(ObjectId::part(ordinal), SlideKind::Slide)),
        },
        Operation::InsertSlide { index, ordinal },
    ])
}

#[test]
fn each_slide_op_applied_then_its_inverse_gives_the_model_back() {
    let start = deck();
    for op in [
        new_slide(&start, 0),
        new_slide(&start, 3),
        Operation::RemoveSlide { index: 1 },
        Operation::MoveSlide { from: 0, to: 2 },
        Operation::MoveSlide { from: 2, to: 0 },
        Operation::SetSlideShow {
            ordinal: 2,
            show: Some(false),
        },
        // A duplicate: a copy of slide 2's part, listed after it.
        Operation::Batch(vec![
            Operation::AddPart {
                entry: Box::new(entry(4, SlideKind::Slide)),
                part: Box::new(SlidePart::new(ObjectId::part(4), SlideKind::Slide)),
            },
            Operation::InsertSlide {
                index: 2,
                ordinal: 4,
            },
        ]),
    ] {
        let mut parts = start.clone();
        let inverse = apply(&mut parts, op.clone()).unwrap();
        assert_ne!(parts, start, "{op:?} changed nothing");
        let again = apply(&mut parts, inverse).unwrap();
        assert_eq!(parts, start, "{op:?} not undone");
        let mut redone = start.clone();
        apply(&mut redone, op.clone()).unwrap();
        let mut via_inverse = start.clone();
        apply(&mut via_inverse, again).unwrap();
        assert_eq!(redone, via_inverse, "{op:?} redo differs");
    }
}

#[test]
fn a_new_slide_undoes_and_redoes_through_history() {
    let mut parts = deck();
    let mut history = History::new();
    let op = new_slide(&parts, 1);
    assert!(op.changes_slide_list());
    history.apply(&mut parts, op).unwrap();
    assert_eq!(parts.structure.slides, vec![1, 4, 2, 3]);
    assert_eq!(parts.parts.len(), 5);
    history.undo(&mut parts).unwrap();
    assert_eq!(parts, deck());
    history.redo(&mut parts).unwrap();
    assert_eq!(parts.structure.slides, vec![1, 4, 2, 3]);
}

#[test]
fn moving_a_slide_keeps_every_other_in_order() {
    let mut parts = deck();
    apply(&mut parts, Operation::MoveSlide { from: 0, to: 2 }).unwrap();
    assert_eq!(parts.structure.slides, vec![2, 3, 1]);
    apply(&mut parts, Operation::MoveSlide { from: 2, to: 1 }).unwrap();
    assert_eq!(parts.structure.slides, vec![2, 1, 3]);
}

#[test]
fn hiding_an_unhidden_slide_and_undoing_leaves_show_absent() {
    let mut parts = deck();
    let inverse = apply(
        &mut parts,
        Operation::SetSlideShow {
            ordinal: 1,
            show: Some(false),
        },
    )
    .unwrap();
    assert_eq!(parts.get(1).unwrap().show, Some(false));
    apply(&mut parts, inverse).unwrap();
    assert_eq!(parts.get(1).unwrap().show, None);
}

#[test]
fn bad_places_and_parts_are_refused_and_change_nothing() {
    let start = deck();
    for (op, code) in [
        (Operation::RemoveSlide { index: 3 }, "CS-TXN-0006"),
        (Operation::MoveSlide { from: 0, to: 3 }, "CS-TXN-0006"),
        (Operation::MoveSlide { from: 9, to: 0 }, "CS-TXN-0006"),
        (
            Operation::InsertSlide {
                index: 9,
                ordinal: 1,
            },
            "CS-TXN-0006",
        ),
        // A layout is not a slide.
        (
            Operation::InsertSlide {
                index: 0,
                ordinal: 0,
            },
            "CS-TXN-0007",
        ),
        (
            Operation::SetSlideShow {
                ordinal: 0,
                show: Some(false),
            },
            "CS-TXN-0007",
        ),
        // A listed part, and one not last, are never dropped.
        (Operation::DropPart { ordinal: 3 }, "CS-TXN-0007"),
        (Operation::DropPart { ordinal: 1 }, "CS-TXN-0007"),
        // An added part takes the next ordinal, or nothing.
        (
            Operation::AddPart {
                entry: Box::new(entry(7, SlideKind::Slide)),
                part: Box::new(SlidePart::new(ObjectId::part(7), SlideKind::Slide)),
            },
            "CS-TXN-0007",
        ),
        (
            Operation::Batch(vec![
                Operation::MoveSlide { from: 0, to: 1 },
                Operation::RemoveSlide { index: 5 },
            ]),
            "CS-TXN-0006",
        ),
    ] {
        let mut parts = start.clone();
        let err = apply(&mut parts, op.clone()).unwrap_err();
        assert_eq!(err.code(), code, "{op:?}");
        assert_eq!(parts, start, "{op:?} left a trace");
    }
}
