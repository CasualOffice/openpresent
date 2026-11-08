//! Undo in a shared session (`COL-04`, `docs/24` §Undo): my own last step, rebased
//! over what others did since.
//!
//! As OpenCalc's collaborative undo (its docs/69): an undo is an ordinary operation,
//! sent and ordered like any edit, so nothing on the wire says it is one. A shared
//! [`History`] keeps each step's undo in a form that travels (`crate::shared`), and
//! each operation that arrives is folded into those steps here with the same
//! transform the server uses: the newest step is rebased over the arrival, the arrival
//! over that step, the next step over what the arrival became, and so on. So an undo
//! takes back what this participant did, where it now is, and leaves what others did.

use core::cell::RefCell;

use casual_slide_model::shape::{ObjectId, ShapeNode};
use casual_slide_model::slide::SlideParts;
use casual_slide_model::text::CharacterProperties;

use super::wire::{Codec, SlideEngine, WireOp};
use super::{Engine, Side};
use crate::{History, Operation};

/// What a step carries beside positions, kept aside while it is transformed: the
/// transform moves it and never reads it.
enum Held {
    Node(Box<ShapeNode>),
    Run(Box<CharacterProperties>),
}

/// A codec that writes a payload as its index in a table, for a round trip through
/// the wire form that never leaves this copy.
#[derive(Default)]
struct Aside(RefCell<Vec<Held>>);

impl Aside {
    fn keep(&self, held: Held) -> String {
        let mut table = self.0.borrow_mut();
        table.push(held);
        (table.len() - 1).to_string()
    }
}

impl Codec for Aside {
    fn node_xml(&self, node: &ShapeNode) -> Option<String> {
        Some(self.keep(Held::Node(Box::new(node.clone()))))
    }

    fn run_xml(&self, properties: &CharacterProperties) -> Option<String> {
        Some(self.keep(Held::Run(Box::new(properties.clone()))))
    }

    fn read_node(&self, xml: &str, id: ObjectId) -> Option<ShapeNode> {
        let index: usize = xml.parse().ok()?;
        match self.0.borrow().get(index)? {
            Held::Node(n) => {
                let mut n = (**n).clone();
                n.id = id;
                Some(n)
            }
            Held::Run(_) => None,
        }
    }

    fn read_run(&self, xml: &str) -> Option<CharacterProperties> {
        let index: usize = xml.parse().ok()?;
        match self.0.borrow().get(index)? {
            Held::Run(r) => Some((**r).clone()),
            Held::Node(_) => None,
        }
    }
}

/// The parent and place of `node` in `parts`.
fn place(parts: &SlideParts, node: ObjectId) -> Option<(ObjectId, usize)> {
    let mut stack = vec![parts.get(node.part)?.shape_tree()?];
    while let Some(at) = stack.pop() {
        if let Some(i) = at.children.iter().position(|c| c.id == node) {
            return Some((at.id, i));
        }
        stack.extend(at.children.iter());
    }
    None
}

/// A step's wire form, `None` when it has none.
fn wire(op: &Operation, parts: &SlideParts, aside: &Aside) -> Option<WireOp> {
    match op {
        Operation::DeleteNode { node } => {
            let (parent, index) = place(parts, *node)?;
            Some(WireOp::DeleteNode {
                node: (*node).into(),
                parent: parent.into(),
                index,
            })
        }
        Operation::Batch(ops) => Some(WireOp::Batch {
            ops: ops
                .iter()
                .map(|o| wire(o, parts, aside))
                .collect::<Option<_>>()?,
        }),
        other => WireOp::of(other, &Operation::Batch(Vec::new()), aside).ok(),
    }
}

/// Fold an operation another participant made, as it was applied to `parts`, into a
/// shared history's undo and redo steps. A step with no wire form is left as it is.
pub fn rebase_history(history: &mut History, parts: &SlideParts, arrived: &WireOp) {
    history.rebase_stacks(&mut |steps: &mut [Operation]| {
        let mut over = arrived.clone();
        for step in steps.iter_mut().rev() {
            if SlideEngine::is_noop(&over) {
                break;
            }
            let aside = Aside::default();
            let Some(mine) = wire(step, parts, &aside) else {
                continue;
            };
            let moved = SlideEngine::transform(&mine, &over, Side::Later);
            over = SlideEngine::transform(&over, &mine, Side::Earlier);
            if let Ok(op) = moved.operation(&aside) {
                *step = op;
            }
        }
    });
}
