// The review operations' tests (`UX-011`): comments, authors, a slide's notes link and a
// shape's name and visibility. Each applied then inverted gives the model back.

use super::*;
use casual_slide_model::comment::{Author, AuthorList, ThreadedComment, ThreadedCommentList};
use casual_slide_model::shape::{NodeKind, NonVisualDrawing, ShapeNode};
use casual_slide_model::slide::{CommonSlideData, DeckStructure, SlideKind};

struct Doc {
    parts: SlideParts,
    review: Review,
}

impl Document for Doc {
    fn parts_mut(&mut self) -> &mut SlideParts {
        &mut self.parts
    }
    fn review_mut(&mut self) -> Option<&mut Review> {
        Some(&mut self.review)
    }
}

fn entry(ordinal: u32, kind: SlideKind) -> PartEntry {
    PartEntry {
        id: ObjectId::part(ordinal),
        kind,
        part: format!("ppt/x/part{ordinal}.xml"),
        layout: None,
        master: None,
        notes: None,
        theme: None,
    }
}

/// A slide at 0 holding one named shape, node 2, and a notes slide at 1 nothing links.
fn doc() -> Doc {
    let mut root = ShapeNode::new(ObjectId { part: 0, node: 1 }, NodeKind::Group);
    let mut shape = ShapeNode::new(ObjectId { part: 0, node: 2 }, NodeKind::Shape);
    shape.non_visual = Some(NonVisualDrawing {
        spid: Some(4),
        name: Some("Title 1".to_owned()),
        ..NonVisualDrawing::default()
    });
    root.children.push(shape);
    let mut slide = SlidePart::new(ObjectId::part(0), SlideKind::Slide);
    slide.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
        controls: Vec::new(),
    });
    let structure = DeckStructure {
        parts: vec![entry(0, SlideKind::Slide), entry(1, SlideKind::Notes)],
        slides: vec![0],
        ..DeckStructure::default()
    };
    Doc {
        parts: SlideParts {
            structure,
            parts: vec![
                Some(slide),
                Some(SlidePart::new(ObjectId::part(1), SlideKind::Notes)),
            ],
        },
        review: Review::default(),
    }
}

fn one_comment() -> ThreadedCommentList {
    ThreadedCommentList {
        comments: vec![ThreadedComment {
            id: Some("{A}".to_owned()),
            author_id: Some("{B}".to_owned()),
            ..ThreadedComment::default()
        }],
    }
}

#[test]
fn each_review_op_applied_then_its_inverse_gives_the_model_back() {
    let renamed = NonVisualDrawing {
        spid: Some(4),
        name: Some("Logo".to_owned()),
        hidden: Some(true),
        ..NonVisualDrawing::default()
    };
    for op in [
        Operation::SetComments {
            ordinal: 0,
            comments: Some(Box::new(one_comment())),
        },
        Operation::SetAuthors {
            authors: Some(Box::new(AuthorList {
                authors: vec![Author {
                    id: Some("{B}".to_owned()),
                    name: Some("Ada".to_owned()),
                    ..Author::default()
                }],
            })),
        },
        Operation::SetNotes {
            ordinal: 0,
            notes: Some(1),
        },
        Operation::SetNonVisual {
            node: ObjectId { part: 0, node: 2 },
            non_visual: Some(Box::new(renamed.clone())),
        },
    ] {
        let mut d = doc();
        let inverse = apply(&mut d, op.clone()).unwrap();
        assert_ne!(
            (&d.parts, &d.review),
            (&doc().parts, &doc().review),
            "{op:?}"
        );
        apply(&mut d, inverse).unwrap();
        assert_eq!(
            (&d.parts, &d.review),
            (&doc().parts, &doc().review),
            "{op:?}"
        );
    }
}

#[test]
fn a_comment_list_set_then_undone_leaves_no_part_behind() {
    let mut d = doc();
    let mut history = History::default();
    history
        .apply(
            &mut d,
            Operation::SetComments {
                ordinal: 0,
                comments: Some(Box::new(one_comment())),
            },
        )
        .unwrap();
    assert_eq!(d.review.comments.get(&0), Some(&one_comment()));
    history.undo(&mut d).unwrap();
    assert!(d.review.comments.is_empty());
    history.redo(&mut d).unwrap();
    assert_eq!(d.review.comments.len(), 1);
}

#[test]
fn a_new_notes_slide_is_added_linked_and_undone_whole() {
    let mut d = doc();
    // Drop the unlinked notes slide first: it is the last part and nothing names it.
    apply(&mut d, Operation::DropPart { ordinal: 1 }).unwrap();
    let notes = SlidePart::new(ObjectId::part(1), SlideKind::Notes);
    let mut history = History::default();
    history
        .apply(
            &mut d,
            Operation::Batch(vec![
                Operation::AddPart {
                    entry: Box::new(entry(1, SlideKind::Notes)),
                    part: Box::new(notes),
                },
                Operation::SetNotes {
                    ordinal: 0,
                    notes: Some(1),
                },
            ]),
        )
        .unwrap();
    assert_eq!(d.parts.structure.entry(0).unwrap().notes, Some(1));
    assert_eq!(d.parts.structure.notes, vec![1]);
    history.undo(&mut d).unwrap();
    assert_eq!(d.parts.parts.len(), 1);
    assert_eq!(d.parts.structure.entry(0).unwrap().notes, None);
    assert!(d.parts.structure.notes.is_empty());
}

#[test]
fn a_linked_notes_slide_is_not_dropped() {
    let mut d = doc();
    apply(
        &mut d,
        Operation::SetNotes {
            ordinal: 0,
            notes: Some(1),
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut d, Operation::DropPart { ordinal: 1 }),
        Err(TxnError::BadPart(1))
    );
}

#[test]
fn what_a_review_op_cannot_name_is_refused_and_changes_nothing() {
    let mut d = doc();
    // A notes link to a part that is not a notes slide, or from one that is not a slide.
    for op in [
        Operation::SetNotes {
            ordinal: 0,
            notes: Some(0),
        },
        Operation::SetNotes {
            ordinal: 1,
            notes: Some(1),
        },
        Operation::SetComments {
            ordinal: 1,
            comments: None,
        },
        // A shape's file id cannot change.
        Operation::SetNonVisual {
            node: ObjectId { part: 0, node: 2 },
            non_visual: Some(Box::new(NonVisualDrawing {
                spid: Some(9),
                ..NonVisualDrawing::default()
            })),
        },
    ] {
        assert!(apply(&mut d, op.clone()).is_err(), "{op:?}");
        assert_eq!(d.parts, doc().parts);
    }
    // A document that keeps no comments.
    let mut bare = doc().parts;
    assert_eq!(
        apply(&mut bare, Operation::SetAuthors { authors: None }),
        Err(TxnError::NoReview)
    );
}
