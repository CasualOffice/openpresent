// Slice 6's tests (`VS-06`), moved from `casual-slide-wasm/src/txn.rs` with its ops.

use super::*;
use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ShapeNode, Transform};
use casual_slide_model::slide::{SlideKind, SlidePart};

fn id(node: u32) -> ObjectId {
    ObjectId { part: 0, node }
}

fn deck() -> SlideParts {
    let mut root = ShapeNode::new(id(1), NodeKind::Group);
    for n in 2..5 {
        let mut s = ShapeNode::new(id(n), NodeKind::Shape);
        s.transform = Some(Transform {
            offset: Some(Point {
                x: Coordinate(i64::from(n) * 100),
                y: Coordinate(0),
            }),
            extent: Some(PositiveSize { cx: 50, cy: 50 }),
            ..Transform::default()
        });
        root.children.push(s);
    }
    let mut part = SlidePart::new(ObjectId::part(0), SlideKind::Slide);
    part.common = Some(casual_slide_model::slide::CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
        controls: Vec::new(),
    });
    SlideParts {
        parts: vec![Some(part)],
        ..SlideParts::default()
    }
}

#[test]
fn each_op_applied_then_its_inverse_gives_the_model_back() {
    let moved = Transform {
        offset: Some(Point {
            x: Coordinate(7),
            y: Coordinate(9),
        }),
        rotation: Some(5_400_000),
        ..Transform::default()
    };
    for op in [
        Operation::SetTransform {
            node: id(3),
            transform: Some(moved),
        },
        Operation::SetTransform {
            node: id(3),
            transform: None,
        },
        Operation::DeleteNode { node: id(2) },
        Operation::DeleteNode { node: id(4) },
        Operation::Batch(vec![
            Operation::SetTransform {
                node: id(3),
                transform: Some(moved),
            },
            Operation::DeleteNode { node: id(3) },
        ]),
    ] {
        let start = deck();
        let mut parts = start.clone();
        let inverse = apply(&mut parts, op.clone()).unwrap();
        assert_ne!(parts, start, "{op:?} changed nothing");
        let again = apply(&mut parts, inverse).unwrap();
        assert_eq!(parts, start, "{op:?} not undone");
        // The inverse of the inverse is the op itself, so redo is exact.
        let mut redone = start.clone();
        apply(&mut redone, op.clone()).unwrap();
        let mut via_inverse = start.clone();
        apply(&mut via_inverse, again).unwrap();
        assert_eq!(redone, via_inverse, "{op:?} redo differs");
    }
}

#[test]
fn a_delete_keeps_z_order_when_undone() {
    let mut parts = deck();
    let mut history = History::new();
    history
        .apply(&mut parts, Operation::DeleteNode { node: id(3) })
        .unwrap();
    let order = |p: &SlideParts| -> Vec<u32> {
        p.get(0)
            .unwrap()
            .shape_tree()
            .unwrap()
            .children
            .iter()
            .map(|c| c.id.node)
            .collect()
    };
    assert_eq!(order(&parts), vec![2, 4]);
    history.undo(&mut parts).unwrap();
    assert_eq!(order(&parts), vec![2, 3, 4]);
    history.redo(&mut parts).unwrap();
    assert_eq!(order(&parts), vec![2, 4]);
}

#[test]
fn a_refused_batch_leaves_no_trace() {
    let start = deck();
    let mut parts = start.clone();
    let err = apply(
        &mut parts,
        Operation::Batch(vec![
            Operation::DeleteNode { node: id(2) },
            Operation::DeleteNode { node: id(99) },
        ]),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0001");
    assert_eq!(parts, start);
    assert_eq!(
        apply(&mut parts, Operation::DeleteNode { node: id(1) })
            .unwrap_err()
            .code(),
        "CS-TXN-0002"
    );
}

#[test]
fn a_step_that_changes_nothing_is_not_an_undo_step() {
    let mut parts = deck();
    let same = parts.get(0).unwrap().node(id(2)).unwrap().transform;
    let mut history = History::new();
    history
        .apply(
            &mut parts,
            Operation::SetTransform {
                node: id(2),
                transform: same,
            },
        )
        .unwrap();
    assert!(!history.can_undo());
}
