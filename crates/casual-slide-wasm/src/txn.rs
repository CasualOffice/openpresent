//! The edit operations slice 6 (`VS-06`) needs, and their undo history.
//!
//! Shaped as `docs/24` gives the op set of `casual-slide-transaction`, and as
//! OpenCalc's `casual-calc-transaction` builds it (PROVENANCE.md, owner instruction
//! 42): every [`apply`] returns the inverse operation, so undo and redo replay
//! inverses and are never a second implementation that can drift. [`History`] is
//! OpenCalc's `History`, cut to what one local editor needs.
//!
//! `casual-slide-transaction` did not exist when this slice started; slice 5
//! (`VS-05`) creates it. These ops move there unchanged when it lands, which is why
//! they live in a module of their own (`VS-06` row).
//!
//! Only the model changes here. The file changes at save, where a transform is
//! patched in place (ADR-013) and a removed node is a writer operation.

use casual_slide_model::shape::{ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::SlideParts;

/// An error applying an operation. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TxnError {
    /// No node has this id.
    NoSuchNode(ObjectId),
    /// The shape tree's own root cannot be removed, nor a node put under a
    /// parent that is not there.
    NoParent(ObjectId),
}

impl TxnError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSuchNode(_) => "CS-TXN-0001",
            Self::NoParent(_) => "CS-TXN-0002",
        }
    }
}

impl core::fmt::Display for TxnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchNode(id) => write!(
                f,
                "[{}] no shape {} in part {}",
                self.code(),
                id.node,
                id.part
            ),
            Self::NoParent(id) => write!(
                f,
                "[{}] shape {} in part {} has no parent to change",
                self.code(),
                id.node,
                id.part
            ),
        }
    }
}

impl std::error::Error for TxnError {}

/// One edit of the model. `docs/24`: a property op sets one property of one
/// object; tree ops insert and delete nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operation {
    /// Properties: set a node's `xfrm`. `None` removes it.
    SetTransform {
        /// The node.
        node: ObjectId,
        /// What it becomes.
        transform: Option<Transform>,
    },
    /// Tree: remove a node and everything under it.
    DeleteNode {
        /// The node.
        node: ObjectId,
    },
    /// Tree: put a node back under `parent` at `index` among its children.
    InsertNode {
        /// The group, or the shape tree's root.
        parent: ObjectId,
        /// Its place, in z-order.
        index: usize,
        /// The node and its subtree.
        node: Box<ShapeNode>,
    },
    /// Several, as one undo step: applied in order, undone in reverse.
    Batch(Vec<Operation>),
}

impl Operation {
    /// The part this operation edits, for a host that shows it. The first, for a batch.
    pub fn part(&self) -> Option<u32> {
        match self {
            Self::SetTransform { node, .. } | Self::DeleteNode { node } => Some(node.part),
            Self::InsertNode { parent, .. } => Some(parent.part),
            Self::Batch(ops) => ops.iter().find_map(Self::part),
        }
    }
}

fn root_mut(parts: &mut SlideParts, part: u32) -> Option<&mut ShapeNode> {
    parts.get_mut(part)?.shape_tree_mut()
}

/// The parent of `id` in the tree under `root`, and the child's index. Iterative.
fn parent_of(root: &mut ShapeNode, id: ObjectId) -> Option<(&mut ShapeNode, usize)> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(at) = node.children.iter().position(|c| c.id == id) {
            return Some((node, at));
        }
        stack.extend(node.children.iter_mut());
    }
    None
}

/// Apply `op` to the model, returning its inverse. A refused operation leaves the
/// model as it was, a batch included.
pub fn apply(parts: &mut SlideParts, op: Operation) -> Result<Operation, TxnError> {
    match op {
        Operation::SetTransform { node, transform } => {
            let shape = parts
                .get_mut(node.part)
                .and_then(|p| p.node_mut(node))
                .ok_or(TxnError::NoSuchNode(node))?;
            let before = core::mem::replace(&mut shape.transform, transform);
            Ok(Operation::SetTransform {
                node,
                transform: before,
            })
        }
        Operation::DeleteNode { node } => {
            let root = root_mut(parts, node.part).ok_or(TxnError::NoSuchNode(node))?;
            if root.id == node {
                return Err(TxnError::NoParent(node));
            }
            let (parent, index) = parent_of(root, node).ok_or(TxnError::NoSuchNode(node))?;
            let removed = parent.children.remove(index);
            Ok(Operation::InsertNode {
                parent: parent.id,
                index,
                node: Box::new(removed),
            })
        }
        Operation::InsertNode {
            parent,
            index,
            node,
        } => {
            let root = root_mut(parts, parent.part).ok_or(TxnError::NoParent(parent))?;
            let holder = root.find_mut(parent).ok_or(TxnError::NoParent(parent))?;
            let at = index.min(holder.children.len());
            let id = node.id;
            holder.children.insert(at, *node);
            Ok(Operation::DeleteNode { node: id })
        }
        Operation::Batch(ops) => {
            let mut inverses = Vec::with_capacity(ops.len());
            for op in ops {
                match apply(parts, op) {
                    Ok(inverse) => inverses.push(inverse),
                    Err(e) => {
                        // Roll back what ran, newest first, so a refusal leaves no trace.
                        for inverse in inverses.into_iter().rev() {
                            let _ = apply(parts, inverse);
                        }
                        return Err(e);
                    }
                }
            }
            inverses.reverse();
            Ok(Operation::Batch(inverses))
        }
    }
}

/// Paired undo and redo stacks over [`apply`], as OpenCalc's `History`.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Operation>,
    redo: Vec<Operation>,
}

impl History {
    /// Apply `op`, recording its inverse for undo and clearing redo. An operation
    /// that changed nothing leaves no undo step, as in OpenCalc: an undo that
    /// undoes nothing looks broken.
    pub fn apply(&mut self, parts: &mut SlideParts, op: Operation) -> Result<(), TxnError> {
        let unchanged = op.clone();
        let inverse = apply(parts, op)?;
        if inverse == unchanged {
            return Ok(());
        }
        self.undo.push(inverse);
        self.redo.clear();
        Ok(())
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Undo the latest step; returns what it applied, or `None` with nothing to undo.
    pub fn undo(&mut self, parts: &mut SlideParts) -> Result<Option<Operation>, TxnError> {
        let Some(op) = self.undo.pop() else {
            return Ok(None);
        };
        let applied = op.clone();
        let inverse = apply(parts, op)?;
        self.redo.push(inverse);
        Ok(Some(applied))
    }

    /// Redo the latest undone step; returns what it applied.
    pub fn redo(&mut self, parts: &mut SlideParts) -> Result<Option<Operation>, TxnError> {
        let Some(op) = self.redo.pop() else {
            return Ok(None);
        };
        let applied = op.clone();
        let inverse = apply(parts, op)?;
        self.undo.push(inverse);
        Ok(Some(applied))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
    use casual_slide_model::shape::NodeKind;
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
        let mut history = History::default();
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
        let mut history = History::default();
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
}
