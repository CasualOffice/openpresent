//! Home > Arrange and Shape Format > Arrange (`UX-006`): Bring to Front, Bring
//! Forward, Send Backward, Send to Back, and Flip Horizontal or Vertical, each one undo
//! step. Order is the shape tree's: a move is a `DeleteNode` and an `InsertNode` of
//! the same node at its new place; the save pass moves the element in the file
//! (`Op::MoveShape`) and changes nothing else.

use std::collections::BTreeSet;

use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_model::shape::ObjectId;
use casual_slide_render::Deck;
use casual_slide_transaction::{Operation, TxnError};

use crate::build::spid;
use crate::{BridgeError, Viewer};

/// Where Arrange moves a shape in z-order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Bring to Front: last, drawn over every other.
    Front,
    /// Bring Forward: over the shape just above it.
    Forward,
    /// Send Backward: under the shape just below it.
    Backward,
    /// Send to Back: first.
    Back,
}

impl Order {
    /// From the bridge's number: 0 front, 1 forward, 2 backward, 3 back.
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::Front,
            1 => Self::Forward,
            2 => Self::Backward,
            3 => Self::Back,
            _ => return None,
        })
    }
}

impl Viewer {
    /// Move shape `node` of slide `index` in z-order among the slide's top-level
    /// shapes. `false` when it is already there.
    ///
    /// # Errors
    ///
    /// `CS-TXN-0001` for no such shape; `CS-WSM-0006` for one inside a group.
    pub fn reorder(&mut self, index: usize, node: u32, order: Order) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let tree = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.shape_tree())
            .ok_or(TxnError::NoSuchObject(id))?;
        let from = tree
            .children
            .iter()
            .position(|c| c.id == id)
            .ok_or(BridgeError::BadArgument)?;
        let last = tree.children.len().saturating_sub(1);
        let to = match order {
            Order::Front => last,
            Order::Forward => from.saturating_add(1).min(last),
            Order::Backward => from.saturating_sub(1),
            Order::Back => 0,
        };
        if to == from {
            return Ok(false);
        }
        let parent = tree.id;
        let shape = tree
            .children
            .get(from)
            .cloned()
            .ok_or(BridgeError::BadArgument)?;
        let changed = self.history.apply(
            self.deck.parts_mut(),
            Operation::Batch(vec![
                Operation::DeleteNode { node: id },
                Operation::InsertNode {
                    parent,
                    index: to,
                    node: Box::new(shape),
                },
            ]),
        )?;
        self.reports.remove(&index);
        Ok(changed)
    }

    /// Flip shape `node` of slide `index`, horizontally or vertically: one undo step.
    ///
    /// # Errors
    ///
    /// `CS-TXN-0001` for no such shape.
    pub fn flip(&mut self, index: usize, node: u32, horizontal: bool) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let parts = self.deck.parts();
        let shape = parts
            .get(ordinal)
            .and_then(|p| p.node(id))
            .ok_or(TxnError::NoSuchObject(id))?;
        // A shape with no transform of its own takes the one it inherits, so the flip
        // does not also move it.
        let mut t = match shape.transform {
            Some(t) => t,
            None => {
                let b = self
                    .shapes(index)?
                    .into_iter()
                    .find(|b| b.node == node)
                    .ok_or(TxnError::NoSuchObject(id))?;
                let op = self.transform_op(index, node, (b.x, b.y, b.cx, b.cy), b.rotation)?;
                match op {
                    Operation::SetTransform {
                        transform: Some(t), ..
                    } => t,
                    _ => return Err(BridgeError::BadArgument),
                }
            }
        };
        let flag = if horizontal {
            &mut t.flip_h
        } else {
            &mut t.flip_v
        };
        // Absent stays absent: flipping back removes the attribute it added.
        *flag = if *flag == Some(true) {
            None
        } else {
            Some(true)
        };
        let changed = self.history.apply(
            self.deck.parts_mut(),
            Operation::SetTransform {
                node: id,
                transform: Some(t),
            },
        )?;
        self.reports.remove(&index);
        Ok(changed)
    }

    /// Whether shape `node` of slide `index` is flipped `(horizontally, vertically)`
    /// by its own transform.
    pub fn shape_flips(&self, index: usize, node: u32) -> (bool, bool) {
        let t = self.slide_ordinal(index).ok().and_then(|part| {
            self.deck
                .parts()
                .get(part)?
                .node(ObjectId { part, node })?
                .transform
        });
        t.map_or((false, false), |t| {
            (t.flip_h == Some(true), t.flip_v == Some(true))
        })
    }

    /// The save pass for z-order: each slide whose top-level shapes the model orders
    /// differently from the file gets them moved, one `Op::MoveShape` a shape, in the
    /// model's order. A slide with a top-level shape without an id is left alone.
    pub(crate) fn order_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let file = Deck::open(bytes.clone())?;
        let model = self.deck.parts();
        let mut plan = SavePlan::new();
        let mut any = false;
        let fparts = file.parts();
        for (i, &o) in fparts.structure.slides.iter().enumerate() {
            let (Some(entry), Some(f), Some(m)) =
                (fparts.structure.entry(o), fparts.get(o), model.slide(i))
            else {
                continue;
            };
            let (Some(ft), Some(mt)) = (f.shape_tree(), m.shape_tree()) else {
                continue;
            };
            let fo: Option<Vec<u32>> = ft.children.iter().map(spid).collect();
            let mo: Option<Vec<u32>> = mt.children.iter().map(spid).collect();
            let (Some(fo), Some(mo)) = (fo, mo) else {
                continue;
            };
            let same_set = fo.iter().collect::<BTreeSet<_>>() == mo.iter().collect::<BTreeSet<_>>();
            if fo == mo || !same_set {
                continue;
            }
            for (index, &shape_id) in mo.iter().enumerate() {
                plan = plan.op(Op::MoveShape {
                    slide: SlideRef::Part(entry.part.clone()),
                    shape_id,
                    index,
                });
            }
            any = true;
        }
        if !any {
            return Ok(bytes);
        }
        Self::run(&plan, bytes)
    }
}
