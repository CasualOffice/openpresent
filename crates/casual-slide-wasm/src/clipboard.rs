//! Cut, Copy, Paste and Duplicate of shapes, and the edits a multiple selection makes
//! at once (`UX-003`): move, align, distribute and delete, each one undo step.
//!
//! Shaped after OpenCalc's clipboard (`webapp/editor.clipboard.js` at `b23c67c`, owner
//! instruction 42): the engine keeps the copied content itself, outside the open
//! document, so a paste reproduces what was copied rather than what the system
//! clipboard's plain text can carry, and it survives File > New and Open, so shapes
//! paste into another deck. The page writes a plain-text stand-in to the system
//! clipboard, as OpenCalc's TSV, and pastes the engine's copy while that stand-in is
//! still what the clipboard holds.
//!
//! What a copy holds is the shape's model, with a picture's bytes beside it. A paste
//! inserts each shape anew, with fresh ids, through the same path Insert uses, so the
//! writer adds it to the file and then patches it to the model (ADR-013).

use std::cell::{Cell, RefCell};

use casual_slide_export::writer::MediaBytes;
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::text::{Bullet, CharacterProperties, TextItem};
use casual_slide_transaction::Operation;

#[cfg(test)]
use crate::build::spid;
use crate::build::{MAX_PICTURE_BYTES, embed, highest, sniff};
use crate::{BridgeError, Viewer};

/// The most shapes one copy holds: PowerPoint's own slide holds no more in practice,
/// and the bound keeps a paste's batch small.
pub const MAX_CLIP_SHAPES: usize = 1000;

/// How far a paste onto the slide it was copied from, and a duplicate, are moved
/// right and down, EMU, so the copy is seen beside its original. Not measured against
/// PowerPoint.
pub const PASTE_OFFSET: i64 = 152_400;

/// One copied shape.
#[derive(Debug, Clone)]
pub struct ClipShape {
    node: ShapeNode,
    picture: Option<MediaBytes>,
}

/// What Copy put on the engine's clipboard.
#[derive(Debug, Clone)]
pub struct Clip {
    /// The deck it came from, as [`Viewer::serial`] numbers it.
    serial: u64,
    /// The part it came from.
    part: u32,
    shapes: Vec<ClipShape>,
}

impl Clip {
    /// How many shapes it holds.
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

thread_local! {
    static NEXT_SERIAL: Cell<u64> = const { Cell::new(0) };
    pub(crate) static CLIPBOARD: RefCell<Option<Clip>> = const { RefCell::new(None) };
}

/// A number no other deck opened in this session has.
pub(crate) fn next_serial() -> u64 {
    NEXT_SERIAL.with(|n| {
        let v = n.get().wrapping_add(1);
        n.set(v);
        v
    })
}

fn props_link(p: Option<&CharacterProperties>) -> bool {
    p.is_some_and(|p| p.hyperlink_click.is_some() || p.hyperlink_mouse_over.is_some())
}

/// Whether the shape names a relationship of its part, other than a picture's own
/// image, which a copy carries as bytes: a link, a picture fill, a picture bullet, or
/// content this model keeps unread. Such a shape pastes only onto its own slide.
fn holds_relationship(node: &ShapeNode) -> bool {
    if node.rel_id.is_some() || matches!(node.properties.fill, Some(Fill::Blip(_))) {
        return true;
    }
    if node
        .non_visual
        .as_ref()
        .is_some_and(|n| n.hyperlink_click.is_some() || n.hyperlink_hover.is_some())
    {
        return true;
    }
    let Some(body) = node.text_body.as_deref() else {
        return false;
    };
    body.paragraphs.iter().any(|p| {
        props_link(p.end_properties.as_deref())
            || p.properties
                .as_deref()
                .is_some_and(|pp| matches!(pp.bullet, Some(Bullet::Picture(_))))
            || p.items.iter().any(|item| match item {
                TextItem::Run(r) => props_link(r.properties.as_deref()),
                TextItem::Break(b) => props_link(b.properties.as_deref()),
                TextItem::Field(f) => props_link(f.properties.as_deref()),
                TextItem::Math(_) | TextItem::Retained(_) => true,
            })
    })
}

impl Viewer {
    /// The shapes `nodes` of slide `index`, copied: refused, before anything is kept,
    /// for one a paste could not yet write (a group, a table, a chart or another frame,
    /// or a picture that is not PNG or JPEG).
    pub fn copy_shapes(&mut self, index: usize, nodes: &[u32]) -> Result<Clip, BridgeError> {
        if nodes.is_empty() || nodes.len() > MAX_CLIP_SHAPES {
            return Err(BridgeError::BadArgument);
        }
        let ordinal = self.slide_ordinal(index)?;
        let mut shapes = Vec::with_capacity(nodes.len());
        // In z-order, as the slide has them, whatever order they were selected in.
        let tree = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.shape_tree())
            .ok_or(BridgeError::BadArgument)?;
        let chain = casual_slide_style::Chain::of(self.deck.parts(), ordinal);
        let mut chosen: Vec<ShapeNode> = Vec::with_capacity(nodes.len());
        for child in &tree.children {
            if nodes.contains(&child.id.node) {
                let mut copy = child.clone();
                // A placeholder is copied as an ordinary shape at the box it is drawn
                // in, with what it inherited from its layout and master written into
                // it, as PowerPoint pastes one where no layout placeholder follows it
                // (`UX-004`): the writer cannot yet add a placeholder, and another
                // slide's layout may have no match.
                if copy.placeholder().is_some() {
                    if let Some(t) = chain.transform(child) {
                        let mut own = copy.transform.unwrap_or_default();
                        own.offset = Some(t.value.offset);
                        own.extent = Some(t.value.extent);
                        copy.transform = Some(own);
                    }
                    self.fold_inherited(child, &chain, &mut copy);
                    if let Some(app) = copy.app.as_mut() {
                        app.placeholder = None;
                    }
                }
                chosen.push(copy);
            }
        }
        if chosen.len() != nodes.len() {
            return Err(BridgeError::BadArgument);
        }
        for node in chosen {
            let picture = match node.kind {
                NodeKind::Shape | NodeKind::Connector => None,
                NodeKind::Picture => Some(self.picture_bytes(ordinal, &node)?),
                _ => return Err(BridgeError::CannotCopy),
            };
            shapes.push(ClipShape { node, picture });
        }
        Ok(Clip {
            serial: self.serial,
            part: ordinal,
            shapes,
        })
    }

    /// Write into `copy` what placeholder `original` draws with from its layout and
    /// master: its text's list style and body properties, folded level by level, and
    /// the fill, line, effects, geometry and style it takes from them. A picture fill
    /// names the layout's or master's relationship, which the slide does not have, so
    /// it is left behind.
    fn fold_inherited(
        &self,
        original: &ShapeNode,
        chain: &casual_slide_style::Chain<'_>,
        copy: &mut ShapeNode,
    ) {
        if let (Some(text), Some(target)) =
            (self.deck.text_chain(original.id), copy.text_body.as_mut())
        {
            let own = original.text_body.as_deref();
            let style = text.folded_list_style();
            let body = text.folded_body(own.and_then(|b| b.properties.as_ref()));
            // Absent stays absent: an empty fold adds no `lstStyle`; `bodyPr` is
            // required, so it is always written.
            if style != Default::default() || target.list_style.is_some() {
                target.list_style = Some(style);
            }
            target.properties = Some(body);
        }
        let inherited = chain.inherited(original);
        for from in [inherited.layout, inherited.master].into_iter().flatten() {
            let (to, p) = (&mut copy.properties, &from.properties);
            if to.fill.is_none() && !matches!(p.fill, Some(Fill::Blip(_))) {
                to.fill.clone_from(&p.fill);
            }
            if to.line.is_none() {
                to.line.clone_from(&p.line);
            }
            if to.effects.is_none() {
                to.effects.clone_from(&p.effects);
            }
            if to.geometry.is_none() {
                to.geometry.clone_from(&p.geometry);
            }
            if copy.style.is_none() {
                copy.style.clone_from(&from.style);
            }
        }
    }

    /// A picture shape's image: one inserted in this session, or the file's part.
    fn picture_bytes(&mut self, ordinal: u32, node: &ShapeNode) -> Result<MediaBytes, BridgeError> {
        let rel = embed(node).ok_or(BridgeError::CannotCopy)?.to_owned();
        if let Some(m) = self.new_media.get(&rel) {
            return Ok(m.clone());
        }
        let name = self
            .deck
            .related_part(ordinal, &rel)
            .ok_or(BridgeError::CannotCopy)?;
        let bytes = self.deck.read_part(&name).ok_or(BridgeError::CannotCopy)?;
        if bytes.len() > MAX_PICTURE_BYTES {
            return Err(BridgeError::CannotCopy);
        }
        let (extension, content_type) = sniff(&bytes).ok_or(BridgeError::CannotCopy)?;
        Ok(MediaBytes {
            bytes,
            extension: extension.to_owned(),
            content_type: content_type.to_owned(),
        })
    }

    /// Paste `clip` onto slide `index`, on top of its shapes, as one undo step; the new
    /// shapes' nodes. Onto the slide it came from, it is moved by [`PASTE_OFFSET`].
    /// A shape naming a relationship of its own slide is refused onto another.
    pub fn paste_shapes(&mut self, index: usize, clip: &Clip) -> Result<Vec<u32>, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let same_slide = clip.serial == self.serial && clip.part == ordinal;
        let offset = if same_slide { PASTE_OFFSET } else { 0 };
        if !same_slide && clip.shapes.iter().any(|s| holds_relationship(&s.node)) {
            return Err(BridgeError::CannotCopy);
        }
        self.insert_copies(ordinal, index, clip, offset)
    }

    /// Duplicate (Ctrl+D): the shapes, copied and pasted beside themselves, leaving the
    /// clipboard as it was.
    pub fn duplicate_shapes(
        &mut self,
        index: usize,
        nodes: &[u32],
    ) -> Result<Vec<u32>, BridgeError> {
        let clip = self.copy_shapes(index, nodes)?;
        self.paste_shapes(index, &clip)
    }

    fn insert_copies(
        &mut self,
        ordinal: u32,
        index: usize,
        clip: &Clip,
        offset: i64,
    ) -> Result<Vec<u32>, BridgeError> {
        // Shared, a pasted picture's bytes cross with it, within the wire's bound.
        for shape in &clip.shapes {
            if let Some(p) = &shape.picture {
                self.shared_media_fits(p.bytes.len())?;
            }
        }
        let part = self
            .deck
            .parts()
            .get(ordinal)
            .ok_or(BridgeError::BadArgument)?;
        let root = part.shape_tree().ok_or(BridgeError::BadArgument)?;
        let parent = root.id;
        let first = root.children.len();
        let (mut top_spid, mut top_node) = highest(part);
        let mut ops = Vec::with_capacity(clip.shapes.len());
        let mut out = Vec::with_capacity(clip.shapes.len());
        let mut media = Vec::new();
        for (at, shape) in (first..).zip(&clip.shapes) {
            let mut node = shape.node.clone();
            // Shared, from this participant's own range, so two pastes never collide.
            match self.shared_mint() {
                Some(m) => (top_spid, top_node) = (m, m),
                None => {
                    top_spid = top_spid.checked_add(1).ok_or(BridgeError::BadArgument)?;
                    top_node = top_node.checked_add(1).ok_or(BridgeError::BadArgument)?;
                }
            }
            node.id = ObjectId {
                part: ordinal,
                node: top_node,
            };
            node.children.clear();
            let nv = node.non_visual.get_or_insert_with(Default::default);
            nv.spid = Some(top_spid);
            // A copy is a new object: PowerPoint's creation id names the original.
            nv.creation_id = None;
            // A lone connector's glue named shapes of the slide it came from.
            node.start_connection = None;
            node.end_connection = None;
            if let Some(t) = node.transform.as_mut()
                && let Some(off) = t.offset.as_mut()
            {
                off.x.0 = off.x.0.saturating_add(offset);
                off.y.0 = off.y.0.saturating_add(offset);
            }
            if let Some(picture) = &shape.picture {
                let (rel, _) = self.mint_media();
                let blip = node
                    .properties
                    .picture
                    .as_mut()
                    .and_then(|p| p.blip.as_mut())
                    .ok_or(BridgeError::CannotCopy)?;
                blip.embed = Some(rel.clone());
                blip.link = None;
                media.push((rel, picture.clone()));
            }
            ops.push(Operation::InsertNode {
                parent,
                index: at,
                node: Box::new(node),
            });
            out.push(top_node);
        }
        for (rel, picture) in media {
            let key = format!("openpresent-new-media/{rel}");
            self.deck
                .add_media(ordinal, &rel, &key, picture.bytes.clone());
            self.new_media.insert(rel, picture);
        }
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(out)
    }

    /// Place several shapes of slide `index` at once, as one undo step: `boxes` holds
    /// `node, x, y, cx, cy, rotation` for each, as [`Viewer::set_transform`] takes them.
    /// Refused, before anything changes, if any one is.
    pub fn set_transforms(&mut self, index: usize, boxes: &[[i64; 6]]) -> Result<(), BridgeError> {
        if boxes.is_empty() || boxes.len() > MAX_CLIP_SHAPES {
            return Err(BridgeError::BadArgument);
        }
        let mut ops = Vec::with_capacity(boxes.len());
        for &[node, x, y, cx, cy, rotation] in boxes {
            let node = u32::try_from(node).map_err(|_| BridgeError::BadArgument)?;
            let rotation = i32::try_from(rotation).map_err(|_| BridgeError::BadArgument)?;
            ops.push(self.transform_op(index, node, (x, y, cx, cy), rotation)?);
        }
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(())
    }

    /// Delete several shapes of slide `index`, as one undo step; refused, before
    /// anything changes, if any one is refused as [`Viewer::delete`] refuses it.
    pub fn delete_shapes(&mut self, index: usize, nodes: &[u32]) -> Result<(), BridgeError> {
        if nodes.is_empty() || nodes.len() > MAX_CLIP_SHAPES {
            return Err(BridgeError::BadArgument);
        }
        let ordinal = self.slide_ordinal(index)?;
        for &node in nodes {
            self.check_removable(ordinal, node)?;
        }
        let ops = nodes
            .iter()
            .map(|&node| Operation::DeleteNode {
                node: ObjectId {
                    part: ordinal,
                    node,
                },
            })
            .collect();
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(())
    }
}

/// Keep `clip` as what Paste pastes.
pub(crate) fn keep(clip: Clip) {
    CLIPBOARD.with(|c| *c.borrow_mut() = Some(clip));
}

/// What Paste pastes, if Copy kept anything.
pub(crate) fn kept() -> Option<Clip> {
    CLIPBOARD.with(|c| c.borrow().clone())
}

/// The spid a pasted shape's original had, for a test that follows it.
#[cfg(test)]
pub(crate) fn clip_spids(clip: &Clip) -> Vec<Option<u32>> {
    clip.shapes.iter().map(|s| spid(&s.node)).collect()
}
