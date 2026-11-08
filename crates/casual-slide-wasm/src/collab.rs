//! The browser's side of co-editing (`COL-04`, `VS-09`): the deck as a [`Replica`], and
//! the shared session over it.
//!
//! Shaped after OpenCalc's `casual-calc-wasm` collaboration calls at `b23c67c`
//! (`collab_begin`, `collab_flush`, `collab_receive`, `collab_acknowledge`, ...), which
//! `webapp/collab.js` drives. While a session is on, the undo history refuses an edit
//! the wire cannot carry ([`shareable`]) and journals every edit it applies, typed
//! text as the runs it became; the journal is turned into wire operations and recorded
//! in the [`ClientSession`] before anything is flushed or received, so local edits are
//! always rebased in order. Each arrival is folded into the undo history too
//! ([`rebase_history`]), so undo takes back this participant's own step where it now is.
//!
//! The wire's XML ([`Codec`]) is written by the writer, as a save writes a new shape:
//! an empty element patched to the model and read back. An arriving node or run is
//! read through the hardened reader, as a file's would be, and a picture's bytes are
//! sniffed and bounded before they become media. A shape or picture this participant
//! inserts takes its node, `cNvPr/@id` and relationship id from a range of its own, so
//! two participants never mint the same one.

use std::collections::BTreeMap;

use casual_slide_export::slide::patch_slide_part;
use casual_slide_export::writer::MediaBytes;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::SlideLimits;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::text::{CharacterProperties, Paragraph, TextBody, TextItem, TextRun};
use casual_slide_ooxml::{OoxmlLimits, Relationship, TargetMode, XmlTree};
use casual_slide_transaction::apply;
use casual_slide_transaction::collab::wire::{
    Codec, Id, MAX_WIRE_MEDIA, MAX_WIRE_MEDIA_TEXT, SlideEngine, WireError, WireMedia, WireOp,
    shareable,
};
use casual_slide_transaction::collab::{
    ClientId, ClientMessage, ClientSession, Engine, Presence, Replica, rebase_history,
};

use crate::build::{NAMESPACES, NEW_REL, TREE_HEAD, embed, read_slide, sniff};
use crate::{BridgeError, Viewer};

/// The most participants whose id ranges fit: each takes 2^20 node numbers and
/// `cNvPr/@id`s, under `u32`'s bound.
pub const MAX_PARTICIPANT: u64 = 4095;

/// A shared session's state in the bridge.
#[derive(Debug)]
pub(crate) struct Shared {
    session: ClientSession<SlideEngine>,
    /// The XML of each node this participant inserted from a template, which its
    /// insert carries as built.
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

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Bytes as standard base64, padded.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18u32, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                let index = usize::try_from((n >> shift) & 63).unwrap_or(0);
                out.push(char::from(BASE64.get(index).copied().unwrap_or(b'A')));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Standard base64 back to bytes; `None` for anything else.
fn unbase64(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let value = |c: u8| BASE64.iter().position(|&b| b == c).map(|v| v as u32);
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for (i, quad) in bytes.chunks(4).enumerate() {
        let last = i + 1 == bytes.len() / 4;
        let pad = quad.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n = 0u32;
        for (j, &c) in quad.iter().enumerate() {
            let v = if j >= 4 - pad { 0 } else { value(c)? };
            n = (n << 6) | v;
        }
        let three = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(three.get(..3 - pad)?);
    }
    Some(out)
}

const WIRE_PART: &str = "ppt/slides/openpresent-shared.xml";

/// Read one node from the XML element on the wire, as the reader reads a slide.
fn read_node(xml: &str, id: ObjectId) -> Option<ShapeNode> {
    let wrapped = format!(
        "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{xml}</p:spTree></p:cSld></p:sld>"
    );
    let part = read_slide(wrapped.as_bytes(), WIRE_PART, id.part).ok()?;
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

/// The element a node is written as: an empty one of its kind, patched to the model
/// by the writer and read back, as a save writes a new shape (ADR-013).
fn write_node(node: &ShapeNode) -> Option<String> {
    let spid = node.non_visual.as_ref().and_then(|n| n.spid).unwrap_or(2);
    let nv = |container: &str, own: &str| {
        format!("<p:{container}><p:cNvPr id=\"{spid}\" name=\"\"/>{own}<p:nvPr/></p:{container}>")
    };
    let body = if node.text_body.is_some() {
        "<p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>"
    } else {
        ""
    };
    let mut rels = Vec::new();
    let skeleton = match node.kind {
        NodeKind::Shape => format!(
            "<p:sp>{}<p:spPr/>{body}</p:sp>",
            nv("nvSpPr", "<p:cNvSpPr/>")
        ),
        NodeKind::Connector => format!(
            "<p:cxnSp>{}<p:spPr/></p:cxnSp>",
            nv("nvCxnSpPr", "<p:cNvCxnSpPr/>")
        ),
        NodeKind::Picture => {
            let rel = embed(node)?;
            rels.push(Relationship {
                id: rel.to_owned(),
                rel_type:
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
                        .to_owned(),
                target: "../media/shared".to_owned(),
                mode: TargetMode::Internal,
            });
            format!(
                "<p:pic>{}<p:blipFill><a:blip r:embed=\"{rel}\"/></p:blipFill><p:spPr/></p:pic>",
                nv("nvPicPr", "<p:cNvPicPr/>")
            )
        }
        _ => return None,
    };
    let xml = format!(
        "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{skeleton}</p:spTree></p:cSld></p:sld>"
    );
    let limits = OoxmlLimits::default();
    let mut tree = XmlTree::parse(xml.as_bytes(), WIRE_PART, &limits).ok()?;
    let mut after = read_slide(xml.as_bytes(), WIRE_PART, node.id.part).ok()?;
    let slot = after.shape_tree_mut()?.children.first_mut()?;
    let mut wanted = node.clone();
    wanted.id = slot.id;
    wanted.children.clear();
    // The skeleton's own non-visual elements, which the schema requires, where the
    // model has none (a run's carrier shape).
    if wanted.non_visual.is_none() {
        wanted.non_visual.clone_from(&slot.non_visual);
    }
    if wanted.app.is_none() {
        wanted.app.clone_from(&slot.app);
    }
    if let (Some(body), Some(skeleton)) =
        (wanted.text_body.as_deref_mut(), slot.text_body.as_deref())
        && body.properties.is_none()
    {
        body.properties.clone_from(&skeleton.properties);
    }
    *slot = wanted;
    patch_slide_part(
        &mut tree,
        WIRE_PART,
        false,
        &rels,
        &after,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &limits,
    )
    .ok()?;
    let out = String::from_utf8(tree.to_bytes()).ok()?;
    let head = "<p:grpSpPr/>";
    let start = out.find(head)?.saturating_add(head.len());
    let end = out.rfind("</p:spTree>")?;
    out.get(start..end).map(str::to_owned)
}

/// Run properties as an `a:rPr` element: a shape holding one run with them, written,
/// and its `rPr` cut out.
fn write_run(properties: &CharacterProperties) -> Option<String> {
    let mut node = ShapeNode::new(ObjectId { part: 0, node: 2 }, NodeKind::Shape);
    node.text_body = Some(Box::new(TextBody {
        paragraphs: vec![Paragraph {
            items: vec![TextItem::Run(TextRun {
                properties: Some(Box::new(properties.clone())),
                text: Some("x".to_owned()),
            })],
            ..Default::default()
        }],
        ..Default::default()
    }));
    let xml = write_node(&node)?;
    let start = xml.find("<a:rPr")?;
    let end = xml.get(start..)?.find("<a:t>")?.saturating_add(start);
    xml.get(start..end).map(str::to_owned)
}

/// Run properties read from an `a:rPr` element, through the reader.
fn read_run(xml: &str) -> Option<CharacterProperties> {
    if !xml.starts_with("<a:rPr") {
        return None;
    }
    let element = format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/>\
         <p:txBody><a:bodyPr/><a:p><a:r>{xml}<a:t>x</a:t></a:r></a:p></p:txBody></p:sp>"
    );
    let node = read_node(&element, ObjectId { part: 0, node: 2 })?;
    let body = node.text_body.as_deref()?;
    let [paragraph] = body.paragraphs.as_slice() else {
        return None;
    };
    match paragraph.items.as_slice() {
        [TextItem::Run(run)] => Some(run.properties.as_deref().cloned().unwrap_or_default()),
        _ => None,
    }
}

/// The bridge's codec: the writer and the reader, the XML a node was inserted from,
/// and the pictures added this session.
struct Wire<'a> {
    xml: &'a BTreeMap<ObjectId, String>,
    media: &'a BTreeMap<String, MediaBytes>,
}

impl Codec for Wire<'_> {
    fn node_xml(&self, node: &ShapeNode) -> Option<String> {
        match self.xml.get(&node.id) {
            Some(xml) => Some(xml.clone()),
            None => write_node(node),
        }
    }

    fn node_media(&self, node: &ShapeNode) -> Option<WireMedia> {
        let rel = embed(node)?;
        let media = self.media.get(rel)?;
        Some(WireMedia {
            rel: rel.to_owned(),
            data: base64(&media.bytes),
        })
    }

    fn run_xml(&self, properties: &CharacterProperties) -> Option<String> {
        write_run(properties)
    }

    fn read_node(&self, xml: &str, id: ObjectId) -> Option<ShapeNode> {
        read_node(xml, id)
    }

    fn read_run(&self, xml: &str) -> Option<CharacterProperties> {
        read_run(xml)
    }
}

/// The deck as a replica of the shared document, with the pictures it was given.
struct Arrival<'a> {
    deck: &'a mut casual_slide_render::Deck,
    media: &'a mut BTreeMap<String, MediaBytes>,
}

impl Arrival<'_> {
    /// Add the pictures an arriving insert carries, as new media of its part. Only a
    /// relationship id of the shape this build mints is taken, and only PNG or JPEG
    /// within the wire's bound.
    fn take_media(&mut self, op: &WireOp) -> Result<(), BridgeError> {
        match op {
            WireOp::InsertNode {
                parent,
                media: Some(m),
                ..
            } => {
                if m.data.len() > MAX_WIRE_MEDIA_TEXT
                    || !m.rel.starts_with(NEW_REL)
                    || m.rel.len() > 64
                    || !m.rel.bytes().all(|b| b.is_ascii_alphanumeric())
                {
                    return Err(BridgeError::Collab(WireError::BadValue.code()));
                }
                let bytes =
                    unbase64(&m.data).ok_or(BridgeError::Collab(WireError::BadValue.code()))?;
                let (extension, content_type) =
                    sniff(&bytes).ok_or(BridgeError::Collab(WireError::BadValue.code()))?;
                let key = format!("openpresent-new-media/{}", m.rel);
                self.deck
                    .add_media(parent.part, &m.rel, &key, bytes.clone());
                self.media.insert(
                    m.rel.clone(),
                    MediaBytes {
                        bytes,
                        extension: extension.to_owned(),
                        content_type: content_type.to_owned(),
                    },
                );
                Ok(())
            }
            WireOp::Batch { ops } => ops.iter().try_for_each(|o| self.take_media(o)),
            _ => Ok(()),
        }
    }
}

impl Replica<SlideEngine> for Arrival<'_> {
    type Error = BridgeError;

    fn apply_remote(&mut self, op: &WireOp) -> Result<(), BridgeError> {
        let (xml, media) = (BTreeMap::new(), BTreeMap::new());
        let codec = Wire {
            xml: &xml,
            media: &media,
        };
        let model = op
            .operation(&codec)
            .map_err(|e| BridgeError::Collab(e.code()))?;
        self.take_media(op)?;
        apply(self.deck.parts_mut(), model)?;
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

    /// A relationship id and media key for a picture added now: in a shared session
    /// from this participant's own range, so two participants' pictures never share
    /// one.
    pub(crate) fn mint_media(&mut self) -> (String, String) {
        self.minted = self.minted.saturating_add(1);
        let tag = match self.shared.as_ref() {
            Some(s) => format!("{}x{}", s.session.id().0, self.minted),
            None => self.minted.to_string(),
        };
        (
            format!("{NEW_REL}{tag}"),
            format!("openpresent-new-media/{NEW_REL}{tag}"),
        )
    }

    /// Refuse, in a shared session, a picture larger than the wire carries.
    pub(crate) fn shared_media_fits(&self, bytes: usize) -> Result<(), BridgeError> {
        if self.is_shared() && bytes > MAX_WIRE_MEDIA {
            return Err(BridgeError::Edit(
                casual_slide_transaction::TxnError::NotShared,
            ));
        }
        Ok(())
    }

    /// Turn what the history applied since last time into wire operations, recorded to
    /// send. An edit with no wire form was refused before it was applied, so this
    /// fails only where the writer could not write a node or a run.
    fn collab_drain(&mut self) -> Result<(), BridgeError> {
        let journal = self.history.take_journal();
        let Some(shared) = self.shared.as_mut() else {
            return Ok(());
        };
        let codec = Wire {
            xml: &shared.xml,
            media: &self.new_media,
        };
        let mut wires = Vec::with_capacity(journal.len());
        for (op, inverse) in journal {
            wires.push(
                WireOp::of(&op, &inverse, &codec).map_err(|e| BridgeError::Collab(e.code()))?,
            );
        }
        for wire in wires {
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
    /// them, at `revision`, and fold each into this participant's undo steps. Returns
    /// the slides they touched, by index, or `-2` when a slide could not be named.
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
            let mut arrival = Arrival {
                deck: &mut self.deck,
                media: &mut self.new_media,
            };
            match shared.session.receive(&mut arrival, op, revision) {
                Ok(applied) => {
                    rebase_history(&mut self.history, self.deck.parts(), &applied);
                    shared.arrived.push(applied);
                }
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
                Arrival {
                    deck: &mut self.deck,
                    media: &mut self.new_media,
                }
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
        // Hiding a slide has no wire form yet: refused before it changes anything.
        assert!(a.set_hidden(0, true).is_err());
        assert_eq!(a.save().unwrap(), before);
        assert!(a.collab_flush().unwrap().is_none(), "nothing was journaled");
    }

    /// Commit everything either has to send, deliver it to the other, until quiet.
    fn sync(
        a: &mut Viewer,
        b: &mut Viewer,
        server: &mut casual_slide_transaction::collab::ServerSession<SlideEngine>,
    ) {
        use casual_slide_transaction::collab::Commit;
        loop {
            let mut moved = false;
            for flip in [false, true] {
                let (me, other) = if flip {
                    (&mut *b, &mut *a)
                } else {
                    (&mut *a, &mut *b)
                };
                let Some(json) = me.collab_flush().unwrap() else {
                    continue;
                };
                moved = true;
                let ClientMessage::Submit(s) =
                    serde_json::from_str::<ClientMessage<SlideEngine>>(&json).unwrap()
                else {
                    panic!("not a submission")
                };
                let Commit::Applied { ops, revision } = server.commit(&s).unwrap() else {
                    panic!("duplicate")
                };
                me.collab_acknowledge(s.seq, revision);
                other
                    .collab_receive(&serde_json::to_string(&ops).unwrap(), revision)
                    .unwrap();
            }
            if !moved {
                break;
            }
        }
    }

    fn pair() -> (
        Viewer,
        Viewer,
        casual_slide_transaction::collab::ServerSession<SlideEngine>,
    ) {
        let (mut a, mut b) = (deck(), deck());
        a.collab_begin(1, 0).unwrap();
        b.collab_begin(2, 0).unwrap();
        (a, b, casual_slide_transaction::collab::ServerSession::new())
    }

    fn typed(v: &mut Viewer, object: ObjectId, at: usize, text: &str) {
        v.edit(
            Operation::InsertText {
                object,
                at,
                text: text.to_owned(),
            },
            true,
        )
        .unwrap();
    }

    /// Size, font, colour and highlight are shared, and text typed after them in one
    /// copy arrives in the other with the run formatting it took (`COL-06`, `COL-01`).
    #[test]
    fn font_size_colour_and_highlight_are_shared_and_typed_text_keeps_its_format() {
        use crate::format::RunSetting;
        let (mut a, mut b, mut server) = pair();
        let (part, node) = title(&a);
        let object = ObjectId { part, node };
        typed(&mut a, object, 0, "Hello world");
        sync(&mut a, &mut b, &mut server);
        a.set_run(object, (0, 5), RunSetting::Size, "40").unwrap();
        a.set_run(object, (0, 5), RunSetting::Font, "Georgia")
            .unwrap();
        b.set_run(object, (6, 11), RunSetting::Color, "srgb C00000")
            .unwrap();
        b.set_run(object, (0, 11), RunSetting::Highlight, "srgb FFFF00")
            .unwrap();
        // Concurrently: Ada types inside her big Georgia word, Bo at the end of his
        // red one.
        typed(&mut a, object, 3, "p");
        typed(&mut b, object, 11, "!");
        sync(&mut a, &mut b, &mut server);
        assert_eq!(a.deck.parts(), b.deck.parts(), "the two models agree");
        assert_eq!(
            a.save().unwrap(),
            b.save().unwrap(),
            "and save the same bytes"
        );
        let parts = a.deck.parts();
        let body = parts
            .get(part)
            .and_then(|p| p.node(object))
            .and_then(|n| n.text_body.as_deref())
            .unwrap();
        let runs: Vec<(String, Option<i32>, bool, bool)> = body.paragraphs[0]
            .items
            .iter()
            .filter_map(|i| match i {
                casual_slide_model::text::TextItem::Run(r) => {
                    let p = r.properties.as_deref().cloned().unwrap_or_default();
                    Some((
                        r.text.clone().unwrap_or_default(),
                        p.size,
                        p.fill.is_some(),
                        p.highlight.is_some(),
                    ))
                }
                _ => None,
            })
            .collect();
        let text: String = runs.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(text, "Helplo world!");
        // The typed "p" took the 40 point Georgia it was typed in; "!" took red.
        let p = runs.iter().find(|r| r.0.contains('p')).unwrap();
        assert_eq!(p.1, Some(4000));
        let bang = runs.iter().find(|r| r.0.contains('!')).unwrap();
        assert!(bang.2 && bang.3, "the ! is red and highlighted: {runs:?}");
    }

    fn png() -> Vec<u8> {
        tiny_skia::Pixmap::new(2, 2).unwrap().encode_png().unwrap()
    }

    /// Paste, Duplicate and Insert Picture in a shared session: the copied node's XML
    /// and the picture's bytes cross, and both copies save the same deck.
    #[test]
    fn paste_duplicate_and_pictures_are_shared_with_their_bytes() {
        let (mut a, mut b, mut server) = pair();
        let (part, node) = title(&a);
        typed(&mut a, ObjectId { part, node }, 0, "Copied");
        let pic = a
            .insert_picture(0, png(), [0, 0, 914_400, 914_400])
            .unwrap();
        sync(&mut a, &mut b, &mut server);
        let dup = b.duplicate_shapes(0, &[node]).unwrap();
        let clip = a.copy_shapes(0, &[pic]).unwrap();
        let pasted = a.paste_shapes(0, &clip).unwrap();
        sync(&mut a, &mut b, &mut server);
        assert!(
            dup[0] >= 2 << 20 && pasted[0] >= 1 << 20,
            "ids from each one's range"
        );
        assert_eq!(a.deck.parts(), b.deck.parts(), "the two models agree");
        let (sa, sb) = (a.save().unwrap(), b.save().unwrap());
        assert_eq!(sa, sb, "and save the same bytes");
        // The picture reached the other copy as media it saves.
        assert!(
            b.new_media.len() >= 2,
            "both pictures are media in the other copy"
        );
    }

    /// Undo of my own typing in a shared session takes back only mine, over what the
    /// other typed since, on both copies.
    #[test]
    fn undo_of_typing_is_shared_and_rebased_over_the_others_edits() {
        let (mut a, mut b, mut server) = pair();
        let (part, node) = title(&a);
        let object = ObjectId { part, node };
        typed(&mut a, object, 0, "world");
        sync(&mut a, &mut b, &mut server);
        a.break_typing();
        typed(&mut a, object, 5, " wide");
        typed(&mut b, object, 0, "Hello ");
        sync(&mut a, &mut b, &mut server);
        assert!(a.undo().is_ok());
        sync(&mut a, &mut b, &mut server);
        let text = |v: &Viewer| {
            v.deck
                .parts()
                .get(part)
                .and_then(|p| p.node(object))
                .and_then(|n| n.text_body.as_deref())
                .map(|b| {
                    b.paragraphs[0]
                        .items
                        .iter()
                        .filter_map(|i| match i {
                            casual_slide_model::text::TextItem::Run(r) => r.text.clone(),
                            _ => None,
                        })
                        .collect::<String>()
                })
                .unwrap()
        };
        assert_eq!(text(&a), "Hello world");
        assert_eq!(text(&b), "Hello world");
        assert_eq!(a.save().unwrap(), b.save().unwrap());
        assert!(a.redo().is_ok());
        sync(&mut a, &mut b, &mut server);
        assert_eq!(text(&b), "Hello world wide");
    }

    #[test]
    fn run_properties_cross_as_xml_and_come_back_the_same() {
        let props = CharacterProperties {
            size: Some(4000),
            bold: Some(true),
            latin: Some(Box::new(casual_slide_model::presentation::TextFont {
                typeface: "Georgia".to_owned(),
                panose: None,
                pitch_family: None,
                charset: None,
            })),
            ..Default::default()
        };
        let xml = write_run(&props).unwrap();
        assert!(xml.starts_with("<a:rPr"), "{xml}");
        assert_eq!(read_run(&xml).unwrap(), props);
        assert!(read_run("<a:rPr><!DOCTYPE x></a:rPr>").is_none());
        assert_eq!(unbase64(&base64(b"hello!?")).unwrap(), b"hello!?");
        assert!(unbase64("a=bc").is_none());
    }

    #[test]
    fn a_node_on_the_wire_must_be_one_leaf_shape() {
        let id = ObjectId { part: 2, node: 9 };
        assert!(read_node("<p:sp><p:nvSpPr><p:cNvPr id=\"9\" name=\"x\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp>", id).is_some());
        assert!(read_node("<!DOCTYPE x [<!ENTITY a \"b\">]><p:sp/>", id).is_none());
        assert!(read_node("", id).is_none());
    }
}
