//! The browser's side of co-editing (`COL-04`, `VS-09`): the deck as a [`Replica`], and
//! the shared session over it.
//!
//! Shaped after OpenCalc's `casual-calc-wasm` collaboration calls at `b23c67c`
//! (`collab_begin`, `collab_flush`, `collab_receive`, `collab_acknowledge`, ...), which
//! `webapp/collab.js` drives. While a session is on, the undo history refuses an edit
//! this slice cannot send ([`shareable`]) and journals every edit it applies; the
//! journal is turned into wire operations and recorded in the [`ClientSession`] before
//! anything is flushed or received, so local edits are always rebased in order.
//!
//! An arriving operation goes through the same `apply` as a local one; an inserted
//! node's XML is read through the hardened reader, as a file's would be. A shape this
//! participant inserts takes its node and `cNvPr/@id` from a range of its own, so two
//! participants never mint the same id.

use std::collections::BTreeMap;

use casual_slide_model::shape::{ObjectId, ShapeNode};
use casual_slide_transaction::collab::wire::{Id, SlideEngine, WireError, WireOp, shareable};
use casual_slide_transaction::collab::{
    ClientId, ClientMessage, ClientSession, Engine, Presence, Replica,
};
use casual_slide_transaction::{apply, typing_takes_after};

use crate::build::{NAMESPACES, TREE_HEAD, read_slide};
use crate::{BridgeError, Viewer};

/// The most participants whose id ranges fit: each takes 2^20 node numbers and
/// `cNvPr/@id`s, under `u32`'s bound.
pub const MAX_PARTICIPANT: u64 = 4095;

/// A shared session's state in the bridge.
#[derive(Debug)]
pub(crate) struct Shared {
    session: ClientSession<SlideEngine>,
    /// The XML of each node this participant inserted, which an insert carries.
    xml: BTreeMap<ObjectId, String>,
    /// The next id this participant mints, in its own range.
    minted: u32,
    /// What the last arrival did, for mapping a caret through it.
    arrived: Vec<WireOp>,
}

impl Shared {
    /// The next node number and `cNvPr/@id`, one value, from this participant's range.
    pub(crate) fn mint(&mut self) -> u32 {
        self.minted = self.minted.saturating_add(1);
        let base = u32::try_from(self.session.id().0).unwrap_or(0) << 20;
        base.saturating_add(self.minted)
    }

    /// Remember the XML a node was built from, so its insert can be sent.
    pub(crate) fn remember(&mut self, node: ObjectId, xml: String) {
        self.xml.insert(node, xml);
    }
}

/// Read one node from the XML element on the wire, as the reader reads a slide.
fn read_node(xml: &str, id: ObjectId) -> Option<ShapeNode> {
    let wrapped = format!(
        "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{xml}</p:spTree></p:cSld></p:sld>"
    );
    let part = read_slide(
        wrapped.as_bytes(),
        "ppt/slides/openpresent-shared.xml",
        id.part,
    )
    .ok()?;
    let root = part.shape_tree()?;
    // Exactly one node, and a leaf: a group's children would need ids of their own.
    let [node] = root.children.as_slice() else {
        return None;
    };
    if !node.children.is_empty() {
        return None;
    }
    let mut node = node.clone();
    node.id = id;
    Some(node)
}

/// The deck as a replica of the shared document.
struct Arrival<'a>(&'a mut casual_slide_render::Deck);

impl Replica<SlideEngine> for Arrival<'_> {
    type Error = BridgeError;

    fn apply_remote(&mut self, op: &WireOp) -> Result<(), BridgeError> {
        let op = op
            .operation(&read_node)
            .map_err(|e| BridgeError::Collab(e.code()))?;
        apply(self.0.parts_mut(), op)?;
        Ok(())
    }

    fn write_snapshot(&self) -> Result<Vec<u8>, BridgeError> {
        Err(BridgeError::Collab(WireError::NotShared.code()))
    }

    fn read_snapshot(&mut self, _bytes: &[u8]) -> Result<(), BridgeError> {
        Err(BridgeError::Collab(WireError::NotShared.code()))
    }
}

impl Viewer {
    /// Start a shared session as participant `client` at `revision`, on the deck as it
    /// stands, which must be the room's (opened from its seed, with its log replayed).
    pub fn collab_begin(&mut self, client: u64, revision: u64) -> Result<(), BridgeError> {
        if client == 0 || client > MAX_PARTICIPANT {
            return Err(BridgeError::Collab("CS-COL-0004"));
        }
        self.typing = None;
        self.history = casual_slide_transaction::History::with_depth(Some(crate::UNDO_DEPTH));
        self.history.share(Some(shareable));
        self.shared = Some(Shared {
            session: ClientSession::new(ClientId(client), revision),
            xml: BTreeMap::new(),
            minted: 0,
            arrived: Vec::new(),
        });
        Ok(())
    }

    /// End the shared session; editing goes on alone.
    pub fn collab_end(&mut self) {
        self.history.share(None);
        self.shared = None;
    }

    /// Whether a shared session is on.
    pub fn is_shared(&self) -> bool {
        self.shared.is_some()
    }

    /// Turn what the history applied since last time into wire operations, recorded to
    /// send. An edit with no wire form was refused before it was applied, so this
    /// fails only for an insert whose XML was not kept, which the bridge prevents.
    fn collab_drain(&mut self) -> Result<(), BridgeError> {
        let journal = self.history.take_journal();
        let Some(shared) = self.shared.as_mut() else {
            return Ok(());
        };
        let parts = self.deck.parts();
        let takes_after = |id: ObjectId, at: usize| {
            parts
                .get(id.part)
                .and_then(|p| p.node(id))
                .and_then(|n| n.text_body.as_deref())
                .is_some_and(|b| typing_takes_after(b, at))
        };
        for (op, inverse) in journal {
            let xml = &shared.xml;
            let wire = WireOp::of(&op, &inverse, &|id| xml.get(&id).cloned(), &takes_after)
                .map_err(|e| BridgeError::Collab(e.code()))?;
            shared.session.record(wire);
        }
        Ok(())
    }

    /// The next submission to send, as the JSON message, if there is one.
    pub fn collab_flush(&mut self) -> Result<Option<String>, BridgeError> {
        self.collab_drain()?;
        let Some(shared) = self.shared.as_mut() else {
            return Ok(None);
        };
        Ok(shared.session.flush().map(|s| {
            serde_json::to_string(&ClientMessage::<SlideEngine>::Submit(s)).unwrap_or_default()
        }))
    }

    /// The server committed this participant's chunks through `through`.
    pub fn collab_acknowledge(&mut self, through: u64, revision: u64) {
        if let Some(shared) = self.shared.as_mut() {
            shared.session.acknowledge(through, revision);
        }
    }

    /// Whether any own edit is not yet acknowledged.
    pub fn collab_unacknowledged(&mut self) -> bool {
        let _ = self.collab_drain();
        self.shared
            .as_ref()
            .is_some_and(|s| s.session.has_unacknowledged())
    }

    /// Apply another participant's operations (a JSON array), as the server ordered
    /// them, at `revision`. Returns the slides they touched, by index, or `-2` when
    /// a slide could not be named.
    pub fn collab_receive(&mut self, ops: &str, revision: u64) -> Result<Vec<i32>, BridgeError> {
        self.collab_drain()?;
        let ops: Vec<WireOp> =
            serde_json::from_str(ops).map_err(|_| BridgeError::Collab("CS-COL-0004"))?;
        let mut shared = self
            .shared
            .take()
            .ok_or(BridgeError::Collab("CS-COL-0002"))?;
        shared.arrived.clear();
        let mut result = Ok(());
        for op in &ops {
            match shared
                .session
                .receive(&mut Arrival(&mut self.deck), op, revision)
            {
                Ok(applied) => shared.arrived.push(applied),
                Err(e) => {
                    result = Err(BridgeError::Collab(e.code()));
                    break;
                }
            }
        }
        let touched = self.touched(&shared.arrived);
        self.shared = Some(shared);
        self.reports.clear();
        self.typing = None;
        result.map(|()| touched)
    }

    /// Replay a room's log on a deck just opened from its seed, before the session.
    pub fn collab_replay(&mut self, ops: &str) -> Result<(), BridgeError> {
        let ops: Vec<WireOp> =
            serde_json::from_str(ops).map_err(|_| BridgeError::Collab("CS-COL-0004"))?;
        for op in &ops {
            if !SlideEngine::is_noop(op) {
                Arrival(&mut self.deck)
                    .apply_remote(op)
                    .map_err(|_| BridgeError::Collab("CS-COL-0002"))?;
            }
        }
        self.reports.clear();
        Ok(())
    }

    /// The slide indexes a list of operations touched.
    fn touched(&self, ops: &[WireOp]) -> Vec<i32> {
        fn parts(op: &WireOp, out: &mut Vec<u32>) {
            match op {
                WireOp::SetTransform {
                    node: Id { part, .. },
                    ..
                }
                | WireOp::InsertNode {
                    parent: Id { part, .. },
                    ..
                }
                | WireOp::DeleteNode {
                    node: Id { part, .. },
                    ..
                }
                | WireOp::InsertText {
                    object: Id { part, .. },
                    ..
                }
                | WireOp::DeleteRange {
                    object: Id { part, .. },
                    ..
                }
                | WireOp::FormatRange {
                    object: Id { part, .. },
                    ..
                } => out.push(*part),
                WireOp::Batch { ops } => ops.iter().for_each(|o| parts(o, out)),
                _ => {}
            }
        }
        let mut ordinals = Vec::new();
        ops.iter().for_each(|o| parts(o, &mut ordinals));
        let slides = &self.deck.parts().structure.slides;
        let mut out: Vec<i32> = ordinals
            .into_iter()
            .map(|o| {
                slides
                    .iter()
                    .position(|&s| s == o)
                    .and_then(|i| i32::try_from(i).ok())
                    .unwrap_or(-2)
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Where a caret at `offset` in a shape's text moves to after the last arrival.
    pub fn collab_map_offset(&self, part: u32, node: u32, offset: usize) -> usize {
        let Some(shared) = self.shared.as_ref() else {
            return offset;
        };
        shared
            .arrived
            .iter()
            .fold(offset, |at, op| op.map_offset(Id { part, node }, at))
    }

    /// This participant's presence as the JSON message to send.
    pub fn collab_presence(presence: Presence) -> String {
        serde_json::to_string(&ClientMessage::<SlideEngine>::Presence(presence)).unwrap_or_default()
    }

    /// The node number and `cNvPr/@id` for a shape inserted now: this participant's
    /// own range in a shared session; `None` alone.
    pub(crate) fn shared_mint(&mut self) -> Option<u32> {
        self.shared.as_mut().map(Shared::mint)
    }

    /// Keep the XML of a node inserted in a shared session, so it can be sent.
    pub(crate) fn shared_remember(&mut self, node: ObjectId, xml: &str) {
        if let Some(shared) = self.shared.as_mut() {
            shared.remember(node, xml.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casual_slide_transaction::Operation;
    use casual_slide_transaction::collab::wire::WireTransform;

    fn deck() -> Viewer {
        Viewer::blank().unwrap()
    }

    fn title(v: &Viewer) -> (u32, u32) {
        let shapes = v.shapes(0).unwrap();
        let s = shapes
            .iter()
            .find(|s| s.flags & crate::ShapeBox::PLACEHOLDER != 0)
            .unwrap();
        (v.slide_part(0).unwrap(), s.node)
    }

    /// Two bridges on the same deck, joined as two participants, each sending what it
    /// edits to the other through a server session: both end with the same saved deck.
    #[test]
    fn two_bridges_exchange_edits_and_save_the_same_deck() {
        use casual_slide_transaction::collab::{Commit, ServerSession, Submission};
        let mut a = deck();
        let mut b = deck();
        a.collab_begin(1, 0).unwrap();
        b.collab_begin(2, 0).unwrap();
        let (part, node) = title(&a);
        a.edit(
            Operation::InsertText {
                object: ObjectId { part, node },
                at: 0,
                text: "Hello".to_owned(),
            },
            true,
        )
        .unwrap();
        b.edit(
            Operation::InsertText {
                object: ObjectId { part, node },
                at: 0,
                text: "Hi ".to_owned(),
            },
            true,
        )
        .unwrap();
        let inserted = b
            .insert_shape(0, crate::Insert::Rectangle, [0, 0, 914_400, 914_400])
            .unwrap();
        a.set_transform(0, node, (100, 100, 2_000_000, 500_000), 0)
            .unwrap();
        let mut server = ServerSession::<SlideEngine>::new();
        let submit = |v: &mut Viewer| -> Submission<SlideEngine> {
            let json = v.collab_flush().unwrap().unwrap();
            match serde_json::from_str::<ClientMessage<SlideEngine>>(&json).unwrap() {
                ClientMessage::Submit(s) => s,
                _ => panic!("not a submission"),
            }
        };
        let sa = submit(&mut a);
        let sb = submit(&mut b);
        let Commit::Applied {
            ops: oa,
            revision: ra,
        } = server.commit(&sa).unwrap()
        else {
            panic!()
        };
        let Commit::Applied {
            ops: ob,
            revision: rb,
        } = server.commit(&sb).unwrap()
        else {
            panic!()
        };
        a.collab_acknowledge(sa.seq, ra);
        b.collab_receive(&serde_json::to_string(&oa).unwrap(), ra)
            .unwrap();
        b.collab_acknowledge(sb.seq, rb);
        a.collab_receive(&serde_json::to_string(&ob).unwrap(), rb)
            .unwrap();
        assert!(!a.collab_unacknowledged() && !b.collab_unacknowledged());
        assert_eq!(a.deck.parts(), b.deck.parts(), "the two models agree");
        assert_eq!(
            a.save().unwrap(),
            b.save().unwrap(),
            "and save the same bytes"
        );
        assert!(
            inserted >= 2 << 20,
            "B's shape took an id from its own range"
        );
        // A third opening the seed and replaying the log reaches the same deck.
        let mut c = deck();
        c.collab_replay(&serde_json::to_string(server.history_since(0).unwrap()).unwrap())
            .unwrap();
        assert_eq!(c.deck.parts(), a.deck.parts());
        let _ = WireTransform::default();
    }

    #[test]
    fn an_edit_a_shared_session_cannot_send_is_refused_and_changes_nothing() {
        let mut a = deck();
        a.collab_begin(1, 0).unwrap();
        let before = a.save().unwrap();
        assert!(a.set_hidden(0, true).is_err());
        // A font size has no wire form (`COL-06`), nor does a pasted shape.
        let (part, node) = title(&a);
        let object = ObjectId { part, node };
        a.edit(
            Operation::InsertText {
                object,
                at: 0,
                text: "x".to_owned(),
            },
            false,
        )
        .unwrap();
        let _ = a.collab_flush().unwrap();
        let typed = a.save().unwrap();
        assert!(
            a.edit(
                Operation::FormatRange {
                    object,
                    start: 0,
                    end: 1,
                    attribute: casual_slide_transaction::Attribute::Size(Some(4000)),
                },
                false
            )
            .is_err()
        );
        assert!(a.duplicate_shapes(0, &[node]).is_err());
        assert_eq!(a.save().unwrap(), typed);
        assert!(a.collab_flush().is_ok(), "nothing unsendable was journaled");
        assert_ne!(typed, before);
    }

    #[test]
    fn a_node_on_the_wire_must_be_one_leaf_shape() {
        let id = ObjectId { part: 2, node: 9 };
        assert!(read_node("<p:sp><p:nvSpPr><p:cNvPr id=\"9\" name=\"x\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp>", id).is_some());
        assert!(read_node("<!DOCTYPE x [<!ENTITY a \"b\">]><p:sp/>", id).is_none());
        assert!(read_node("", id).is_none());
    }
}
