//! Slides' operations on the wire, and their transform (`docs/24` §The op set, §Text OT).
//!
//! The wire carries the Text family (insert, delete, and format with every attribute
//! the Home tab's Font group sets), a shape's transform from Properties, and insert
//! and delete of a top-level node from Tree, a picture's bytes with it. Every other
//! operation is refused in a shared session before it is applied ([`shareable`]), so
//! two copies never part because one edit could not be sent.
//!
//! Conflict rules, as `docs/24` states them:
//! - **Properties**: per property, the server's order wins: of two concurrent
//!   transforms of one shape, the later stands and the earlier becomes nothing.
//! - **Text**: character OT inside one body. Positions count a paragraph break as one
//!   character. Typed text carries the run properties it took where it was typed
//!   (`COL-01`), so no copy formats it from a neighbour. A format over a concurrently
//!   deleted range formats what remains; text typed strictly inside a formatted range
//!   is formatted with it; text typed inside a deleted range stays, at the cut.
//! - **Tree**: a node's place among its parent's children is an index, transformed as
//!   a list; a delete beats a concurrent edit of the node, which becomes nothing.
//!
//! A node crosses the wire as its own XML element, which the receiving client reads
//! through the same hardened reader as a file (`casual-slide-import`), so nothing on
//! the wire is trusted further than a deck is.

use casual_slide_model::presentation::{
    COORDINATE_MIN, Coordinate, POSITIVE_COORDINATE_MAX, Point, PositiveSize,
};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{ObjectId, ShapeNode, Transform};
use casual_slide_model::text::{CharacterProperties, UnderlineType};
use serde::{Deserialize, Serialize};

use super::{Engine, Side};
use crate::{Attribute, Operation};

/// The protocol version of slides' collaboration. 1 was `VS-09`'s first slice; 2
/// carries typed text's run properties, every font attribute, and pictures' bytes.
pub const PROTOCOL_VERSION: u32 = 2;

/// The most characters one inserted text may carry.
pub const MAX_WIRE_TEXT: usize = 1 << 20;

/// The most bytes a node's or a run's XML may carry.
pub const MAX_WIRE_XML: usize = 1 << 20;

/// The most bytes of a picture one inserted node may carry: its base64 then fits a
/// frame ([`super::MAX_FRAME_BYTES`]) with room for the rest of the message.
pub const MAX_WIRE_MEDIA: usize = 32 * 1024 * 1024;

/// [`MAX_WIRE_MEDIA`] as base64 characters.
pub const MAX_WIRE_MEDIA_TEXT: usize = 44_739_244;
const _: () = assert!(MAX_WIRE_MEDIA_TEXT == MAX_WIRE_MEDIA.div_ceil(3) * 4);

/// The most formats one inserted text may have picked up; one per attribute kind.
const MAX_WIRE_FORMATS: usize = 7;

/// A node, by its part's ordinal and its number in the part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id {
    /// The part's ordinal.
    pub part: u32,
    /// The node.
    pub node: u32,
}

impl From<ObjectId> for Id {
    fn from(o: ObjectId) -> Self {
        Self {
            part: o.part,
            node: o.node,
        }
    }
}

impl From<Id> for ObjectId {
    fn from(i: Id) -> Self {
        Self {
            part: i.part,
            node: i.node,
        }
    }
}

/// A shape's `xfrm` on the wire; every field optional, as in the file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct WireTransform {
    /// `off`.
    pub x: Option<i64>,
    /// `off`.
    pub y: Option<i64>,
    /// `ext`.
    pub cx: Option<i64>,
    /// `ext`.
    pub cy: Option<i64>,
    /// `chOff`.
    pub ch_x: Option<i64>,
    /// `chOff`.
    pub ch_y: Option<i64>,
    /// `chExt`.
    pub ch_cx: Option<i64>,
    /// `chExt`.
    pub ch_cy: Option<i64>,
    /// `@rot`.
    pub rot: Option<i32>,
    /// `@flipH`.
    pub flip_h: Option<bool>,
    /// `@flipV`.
    pub flip_v: Option<bool>,
}

fn point(x: Option<i64>, y: Option<i64>) -> Result<Option<Point>, WireError> {
    let ok = |v: i64| (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&v);
    match (x, y) {
        (None, None) => Ok(None),
        (Some(x), Some(y)) if ok(x) && ok(y) => Ok(Some(Point {
            x: Coordinate(x),
            y: Coordinate(y),
        })),
        _ => Err(WireError::BadValue),
    }
}

fn size(cx: Option<i64>, cy: Option<i64>) -> Result<Option<PositiveSize>, WireError> {
    let ok = |v: i64| (0..=POSITIVE_COORDINATE_MAX).contains(&v);
    match (cx, cy) {
        (None, None) => Ok(None),
        (Some(cx), Some(cy)) if ok(cx) && ok(cy) => Ok(Some(PositiveSize { cx, cy })),
        _ => Err(WireError::BadValue),
    }
}

impl WireTransform {
    fn of(t: &Transform) -> Self {
        Self {
            x: t.offset.map(|p| p.x.0),
            y: t.offset.map(|p| p.y.0),
            cx: t.extent.map(|s| s.cx),
            cy: t.extent.map(|s| s.cy),
            ch_x: t.child_offset.map(|p| p.x.0),
            ch_y: t.child_offset.map(|p| p.y.0),
            ch_cx: t.child_extent.map(|s| s.cx),
            ch_cy: t.child_extent.map(|s| s.cy),
            rot: t.rotation,
            flip_h: t.flip_h,
            flip_v: t.flip_v,
        }
    }

    fn model(&self) -> Result<Transform, WireError> {
        if self
            .rot
            .is_some_and(|r| !(-21_600_000..=21_600_000).contains(&r))
        {
            return Err(WireError::BadValue);
        }
        Ok(Transform {
            offset: point(self.x, self.y)?,
            extent: size(self.cx, self.cy)?,
            child_offset: point(self.ch_x, self.ch_y)?,
            child_extent: size(self.ch_cx, self.ch_cy)?,
            rotation: self.rot,
            flip_h: self.flip_h,
            flip_v: self.flip_v,
        })
    }
}

/// How the wire's XML is written and read: by the host, which has the writer and the
/// hardened reader. A node crosses as its element, run properties as an `a:rPr`
/// element; neither is trusted further than a deck is.
pub trait Codec {
    /// The element a node is written as, to send it.
    fn node_xml(&self, node: &ShapeNode) -> Option<String>;
    /// The picture a node shows, when the receiver does not have it: its relationship
    /// id as the node's XML names it, and its bytes in base64.
    fn node_media(&self, node: &ShapeNode) -> Option<WireMedia> {
        let _ = node;
        None
    }
    /// Run properties as an `a:rPr` element.
    fn run_xml(&self, properties: &CharacterProperties) -> Option<String>;
    /// A node read from the wire, numbered `id`; `None` if it is not one node.
    fn read_node(&self, xml: &str, id: ObjectId) -> Option<ShapeNode>;
    /// Run properties read from an `a:rPr` element.
    fn read_run(&self, xml: &str) -> Option<CharacterProperties>;
}

/// A character attribute on the wire. Size is a number; a font, a colour and a
/// highlight are run properties holding that one element, as `a:rPr` XML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum WireAttribute {
    /// `@b`; `None` removes it.
    Bold(Option<bool>),
    /// `@i`.
    Italic(Option<bool>),
    /// `@u`, as its token.
    Underline(Option<String>),
    /// `@sz`, hundredths of a point.
    Size(Option<i32>),
    /// `latin`, in an `a:rPr`.
    Latin(Option<String>),
    /// The fill, in an `a:rPr`: Font Color.
    Fill(Option<String>),
    /// `highlight`, in an `a:rPr`.
    Highlight(Option<String>),
}

impl WireAttribute {
    fn same_kind(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }

    fn of(attribute: &Attribute, codec: &dyn Codec) -> Result<Self, WireError> {
        let xml = |p: CharacterProperties| codec.run_xml(&p).ok_or(WireError::NotShared);
        Ok(match attribute {
            Attribute::Bold(v) => Self::Bold(*v),
            Attribute::Italic(v) => Self::Italic(*v),
            Attribute::Underline(v) => Self::Underline(v.as_ref().map(Lexical::to_lexical)),
            Attribute::Size(v) => Self::Size(*v),
            Attribute::Latin(v) => Self::Latin(
                v.as_ref()
                    .map(|f| {
                        xml(CharacterProperties {
                            latin: Some(f.clone()),
                            ..Default::default()
                        })
                    })
                    .transpose()?,
            ),
            Attribute::Fill(v) => Self::Fill(
                v.as_ref()
                    .map(|f| {
                        xml(CharacterProperties {
                            fill: Some(f.clone()),
                            ..Default::default()
                        })
                    })
                    .transpose()?,
            ),
            Attribute::Highlight(v) => Self::Highlight(
                v.as_ref()
                    .map(|c| {
                        xml(CharacterProperties {
                            highlight: Some(Some(c.clone())),
                            ..Default::default()
                        })
                    })
                    .transpose()?,
            ),
        })
    }

    fn model(&self, codec: &dyn Codec) -> Result<Attribute, WireError> {
        let read = |xml: &str| {
            if xml.len() > MAX_WIRE_XML {
                return Err(WireError::BadValue);
            }
            codec.read_run(xml).ok_or(WireError::BadNode)
        };
        Ok(match self {
            Self::Bold(v) => Attribute::Bold(*v),
            Self::Italic(v) => Attribute::Italic(*v),
            Self::Underline(None) => Attribute::Underline(None),
            Self::Underline(Some(t)) => Attribute::Underline(Some(
                UnderlineType::parse_lexical(t).ok_or(WireError::BadValue)?,
            )),
            Self::Size(v) => {
                if v.is_some_and(|s| !(100..=400_000).contains(&s)) {
                    return Err(WireError::BadValue);
                }
                Attribute::Size(*v)
            }
            Self::Latin(v) => Attribute::Latin(match v {
                Some(x) => Some(read(x)?.latin.ok_or(WireError::BadValue)?),
                None => None,
            }),
            Self::Fill(v) => Attribute::Fill(match v {
                Some(x) => Some(read(x)?.fill.ok_or(WireError::BadValue)?),
                None => None,
            }),
            Self::Highlight(v) => Attribute::Highlight(match v {
                Some(x) => Some(read(x)?.highlight.flatten().ok_or(WireError::BadValue)?),
                None => None,
            }),
        })
    }
}

/// A picture's bytes, carried with the node that shows them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WireMedia {
    /// The relationship id the node's `a:blip/@r:embed` names.
    pub rel: String,
    /// The bytes, base64.
    pub data: String,
}

/// One operation as it travels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum WireOp {
    /// Nothing: what an operation becomes when a concurrent one made it moot.
    Noop,
    /// Set a node's `xfrm`; `None` removes it.
    SetTransform {
        /// The node.
        node: Id,
        /// What it becomes.
        transform: Option<WireTransform>,
    },
    /// Put a node, given as its XML element, under `parent` at `index`.
    InsertNode {
        /// The group, or the shape tree's root.
        parent: Id,
        /// Its place among the parent's children, in z-order.
        index: usize,
        /// The node's number.
        node: Id,
        /// The element: `p:sp`, `p:cxnSp`, `p:pic`.
        xml: String,
        /// A picture's bytes, which the receiver adds as new media.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        media: Option<WireMedia>,
    },
    /// Remove a node, which was at `index` under `parent`.
    DeleteNode {
        /// The node.
        node: Id,
        /// Its parent.
        parent: Id,
        /// Its place there.
        index: usize,
    },
    /// Insert typed text into a shape's body, with the run properties it had where
    /// it was typed (`COL-01`): it never takes them from a neighbour here.
    InsertText {
        /// The shape.
        object: Id,
        /// Where, in characters.
        at: usize,
        /// What.
        text: String,
        /// Its run properties as an `a:rPr` element; empty for none.
        run: String,
        /// Formats it was given by concurrent formats it was typed inside, set over
        /// `run` in order.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        formats: Vec<WireAttribute>,
    },
    /// Delete `[start, end)` of a shape's body.
    DeleteRange {
        /// The shape.
        object: Id,
        /// First character deleted.
        start: usize,
        /// First kept.
        end: usize,
    },
    /// Format `[start, end)` of a shape's body.
    FormatRange {
        /// The shape.
        object: Id,
        /// First character formatted.
        start: usize,
        /// First not.
        end: usize,
        /// What to set.
        attribute: WireAttribute,
    },
    /// Several, in order.
    Batch {
        /// The operations.
        ops: Vec<WireOp>,
    },
}

/// Why a wire operation was refused. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WireError {
    /// An operation slides cannot share yet (`CS-COL-0003`).
    NotShared,
    /// A value outside what the file format allows (`CS-COL-0004`).
    BadValue,
    /// A node's XML could not be read as one node (`CS-COL-0005`).
    BadNode,
}

impl WireError {
    /// The stable diagnostic code (`docs/20`).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::NotShared => "CS-COL-0003",
            Self::BadValue => "CS-COL-0004",
            Self::BadNode => "CS-COL-0005",
        }
    }
}

impl core::fmt::Display for WireError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let what = match self {
            Self::NotShared => "this edit cannot be shared yet",
            Self::BadValue => "a value is outside what the format allows",
            Self::BadNode => "a node's XML is not one node",
        };
        write!(f, "{}: {what}", self.code())
    }
}

impl std::error::Error for WireError {}

/// Whether `op` can be shared, so a shared session may apply it.
#[must_use]
pub fn shareable(op: &Operation) -> bool {
    match op {
        Operation::SetTransform { .. }
        | Operation::DeleteNode { .. }
        | Operation::InsertNode { .. }
        | Operation::InsertText { .. }
        | Operation::InsertRun { .. }
        | Operation::DeleteRange { .. }
        | Operation::FormatRange { .. } => true,
        Operation::Batch(ops) => ops.iter().all(shareable),
        _ => false,
    }
}

impl WireOp {
    /// The wire form of `op`, applied locally with `inverse` returned, as a shared
    /// [`crate::History`] journals them: typed text as [`Operation::InsertRun`]. A
    /// delete takes its parent and place from its inverse.
    pub fn of(op: &Operation, inverse: &Operation, codec: &dyn Codec) -> Result<Self, WireError> {
        Ok(match (op, inverse) {
            (Operation::SetTransform { node, transform }, _) => Self::SetTransform {
                node: (*node).into(),
                transform: transform.as_ref().map(WireTransform::of),
            },
            (
                Operation::InsertNode {
                    parent,
                    index,
                    node,
                },
                _,
            ) => Self::InsertNode {
                parent: (*parent).into(),
                index: *index,
                node: node.id.into(),
                xml: codec.node_xml(node).ok_or(WireError::NotShared)?,
                media: codec.node_media(node),
            },
            (Operation::DeleteNode { node }, Operation::InsertNode { parent, index, .. }) => {
                Self::DeleteNode {
                    node: (*node).into(),
                    parent: (*parent).into(),
                    index: *index,
                }
            }
            (
                Operation::InsertRun {
                    object,
                    at,
                    text,
                    properties,
                },
                _,
            ) => Self::InsertText {
                object: (*object).into(),
                at: *at,
                text: text.clone(),
                run: match properties {
                    Some(p) => codec.run_xml(p).ok_or(WireError::NotShared)?,
                    None => String::new(),
                },
                formats: Vec::new(),
            },
            (Operation::DeleteRange { object, start, end }, _) => Self::DeleteRange {
                object: (*object).into(),
                start: *start,
                end: *end,
            },
            (
                Operation::FormatRange {
                    object,
                    start,
                    end,
                    attribute,
                },
                _,
            ) => Self::FormatRange {
                object: (*object).into(),
                start: *start,
                end: *end,
                attribute: WireAttribute::of(attribute, codec)?,
            },
            (Operation::Batch(ops), inverse) => {
                // A batch's inverses run newest first. Typed text cut into runs is a
                // batch whose inverse is one restore, which none of its runs needs.
                let none = Operation::Batch(Vec::new());
                let inverses: Vec<&Operation> = match inverse {
                    Operation::Batch(inverses) => inverses.iter().rev().collect(),
                    _ => Vec::new(),
                };
                let mut out = Vec::with_capacity(ops.len());
                for (i, op) in ops.iter().enumerate() {
                    let inverse = inverses.get(i).copied().unwrap_or(&none);
                    out.push(Self::of(op, inverse, codec)?);
                }
                Self::Batch { ops: out }
            }
            // Typed text crosses with its run properties, as `InsertRun`; a shared
            // history sends it so.
            _ => return Err(WireError::NotShared),
        })
    }

    /// The model operation this wire operation is, reading XML with `codec`. Values
    /// are checked; text, XML and media are bounded; nothing is trusted.
    pub fn operation(&self, codec: &dyn Codec) -> Result<Operation, WireError> {
        Ok(match self {
            Self::Noop => Operation::Batch(Vec::new()),
            Self::SetTransform { node, transform } => Operation::SetTransform {
                node: (*node).into(),
                transform: transform.as_ref().map(WireTransform::model).transpose()?,
            },
            Self::InsertNode {
                parent,
                index,
                node,
                xml,
                media,
            } => {
                if xml.len() > MAX_WIRE_XML
                    || media
                        .as_ref()
                        .is_some_and(|m| m.data.len() > MAX_WIRE_MEDIA_TEXT)
                {
                    return Err(WireError::BadValue);
                }
                let id: ObjectId = (*node).into();
                if id.part != parent.part || id.node < 2 {
                    return Err(WireError::BadValue);
                }
                let mut read = codec.read_node(xml, id).ok_or(WireError::BadNode)?;
                read.id = id;
                Operation::InsertNode {
                    parent: (*parent).into(),
                    index: *index,
                    node: Box::new(read),
                }
            }
            Self::DeleteNode { node, .. } => Operation::DeleteNode {
                node: (*node).into(),
            },
            Self::InsertText {
                object,
                at,
                text,
                run,
                formats,
            } => {
                if text.chars().count() > MAX_WIRE_TEXT
                    || run.len() > MAX_WIRE_XML
                    || formats.len() > MAX_WIRE_FORMATS
                {
                    return Err(WireError::BadValue);
                }
                let mut properties = if run.is_empty() {
                    None
                } else {
                    Some(Box::new(codec.read_run(run).ok_or(WireError::BadNode)?))
                };
                for f in formats {
                    f.model(codec)?
                        .set(properties.get_or_insert_with(Box::default));
                }
                Operation::InsertRun {
                    object: (*object).into(),
                    at: *at,
                    text: text.clone(),
                    properties,
                }
            }
            Self::DeleteRange { object, start, end } => {
                if start > end {
                    return Err(WireError::BadValue);
                }
                Operation::DeleteRange {
                    object: (*object).into(),
                    start: *start,
                    end: *end,
                }
            }
            Self::FormatRange {
                object,
                start,
                end,
                attribute,
            } => {
                if start > end {
                    return Err(WireError::BadValue);
                }
                Operation::FormatRange {
                    object: (*object).into(),
                    start: *start,
                    end: *end,
                    attribute: attribute.model(codec)?,
                }
            }
            Self::Batch { ops } => Operation::Batch(
                ops.iter()
                    .map(|op| op.operation(codec))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    /// The text body this operation edits, if it is a text operation.
    fn text_object(&self) -> Option<Id> {
        match self {
            Self::InsertText { object, .. }
            | Self::DeleteRange { object, .. }
            | Self::FormatRange { object, .. } => Some(*object),
            _ => None,
        }
    }

    /// The node this operation edits or removes, if any.
    fn target(&self) -> Option<Id> {
        match self {
            Self::SetTransform { node, .. } | Self::DeleteNode { node, .. } => Some(*node),
            _ => self.text_object(),
        }
    }

    /// Where a caret at `offset` in `object` moves to once this operation is applied.
    #[must_use]
    pub fn map_offset(&self, object: Id, offset: usize) -> usize {
        match self {
            Self::InsertText {
                object: o,
                at,
                text,
                ..
            } if *o == object && *at <= offset => offset.saturating_add(text_length(text)),
            Self::DeleteRange {
                object: o,
                start,
                end,
            } if *o == object => shift_over_delete(offset, *start, *end),
            Self::Batch { ops } => ops.iter().fold(offset, |at, op| op.map_offset(object, at)),
            _ => offset,
        }
    }
}

/// How many characters `text` adds to a body: what the model keeps of it, a line feed
/// and a vertical tab one each (`InsertText`).
#[must_use]
pub fn text_length(text: &str) -> usize {
    text.chars()
        .filter(|&c| c == '\n' || c == '\u{b}' || crate::keep_char(c))
        .count()
}

/// `x` after `[start, end)` is deleted.
fn shift_over_delete(x: usize, start: usize, end: usize) -> usize {
    x - (x.min(end) - x.min(start))
}

/// `[start, end)` with `[s, e)` taken out, as up to two ranges in the coordinates
/// before the cut, the later first so the earlier stays valid when applied in order.
fn subtract(start: usize, end: usize, s: usize, e: usize) -> Vec<(usize, usize)> {
    if e <= start || s >= end || s >= e {
        return vec![(start, end)];
    }
    let mut out = Vec::new();
    if e < end {
        out.push((e, end));
    }
    if start < s {
        out.push((start, s));
    }
    out
}

fn batch_or_single(mut ops: Vec<WireOp>) -> WireOp {
    ops.retain(|op| !SlideEngine::is_noop(op));
    match ops.len() {
        0 => WireOp::Noop,
        1 => ops.remove(0),
        _ => WireOp::Batch { ops },
    }
}

/// Slides' collaboration engine: [`WireOp`] and its transform.
#[derive(Debug, Clone, Copy, Default)]
pub struct SlideEngine;

impl Engine for SlideEngine {
    type Op = WireOp;
    const PROTOCOL: u32 = PROTOCOL_VERSION;

    fn is_noop(op: &WireOp) -> bool {
        match op {
            WireOp::Noop => true,
            WireOp::Batch { ops } => ops.iter().all(Self::is_noop),
            WireOp::InsertText { text, .. } => text.is_empty(),
            WireOp::DeleteRange { start, end, .. } | WireOp::FormatRange { start, end, .. } => {
                start >= end
            }
            _ => false,
        }
    }

    fn transform(op: &WireOp, over: &WireOp, side: Side) -> WireOp {
        transform(op, over, side)
    }
}

fn transform(op: &WireOp, over: &WireOp, side: Side) -> WireOp {
    use WireOp as W;
    if SlideEngine::is_noop(op) {
        return W::Noop;
    }
    // A batch over anything: each member over `over`, which moves on past each.
    if let W::Batch { ops } = op {
        let mut over = over.clone();
        let mut out = Vec::with_capacity(ops.len());
        for member in ops {
            out.push(transform(member, &over, side));
            over = transform(&over, member, side.flip());
        }
        return batch_or_single(out);
    }
    // Anything over a batch: over each member in turn.
    if let W::Batch { ops: overs } = over {
        let mut op = op.clone();
        for member in overs {
            op = transform(&op, member, side);
        }
        return op;
    }
    match (op, over) {
        (W::Noop, _) => W::Noop,
        (_, W::Noop) => op.clone(),

        // Tree: a delete beats a concurrent edit of the node, and of its children.
        (W::DeleteNode { node, .. }, W::DeleteNode { node: gone, .. }) if node == gone => W::Noop,
        (_, W::DeleteNode { node: gone, .. }) if op.target() == Some(*gone) => W::Noop,
        (W::InsertNode { parent, .. }, W::DeleteNode { node: gone, .. }) if parent == gone => {
            W::Noop
        }
        // Tree: a parent's children as a list.
        (
            W::InsertNode { parent, index, .. },
            W::InsertNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => {
            let mut out = op.clone();
            if let W::InsertNode { index: slot, .. } = &mut out
                && (*i2 < *index || (*i2 == *index && side == Side::Later))
            {
                *slot = index + 1;
            }
            out
        }
        (
            W::InsertNode { parent, index, .. },
            W::DeleteNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => {
            let mut out = op.clone();
            if let W::InsertNode { index: slot, .. } = &mut out
                && *i2 < *index
            {
                *slot = index - 1;
            }
            out
        }
        (
            W::DeleteNode {
                node,
                parent,
                index,
            },
            W::InsertNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => W::DeleteNode {
            node: *node,
            parent: *parent,
            index: if *i2 <= *index { index + 1 } else { *index },
        },
        (
            W::DeleteNode {
                node,
                parent,
                index,
            },
            W::DeleteNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => W::DeleteNode {
            node: *node,
            parent: *parent,
            index: if *i2 < *index { index - 1 } else { *index },
        },

        // Properties: per property, the server's order wins.
        (W::SetTransform { node, .. }, W::SetTransform { node: n2, .. }) if node == n2 => {
            match side {
                Side::Later => op.clone(),
                Side::Earlier => W::Noop,
            }
        }

        // Text: character OT inside one body.
        _ if op.text_object().is_some() && op.text_object() == over.text_object() => {
            transform_text(op, over, side)
        }
        _ => op.clone(),
    }
}

/// Text OT. Typed text carries its own run properties, so where it lands decides
/// nothing about its format: text typed inside a concurrently deleted range survives
/// at the cut, and text typed strictly inside a concurrently formatted range takes
/// that format, as typing there would; at a range's edge it does not.
fn transform_text(op: &WireOp, over: &WireOp, side: Side) -> WireOp {
    use WireOp as W;
    let with_at = |at: usize| {
        let mut out = op.clone();
        if let W::InsertText { at: slot, .. } = &mut out {
            *slot = at;
        }
        out
    };
    let format = |object: Id, start: usize, end: usize, attribute: &WireAttribute| W::FormatRange {
        object,
        start,
        end,
        attribute: attribute.clone(),
    };
    match (op, over) {
        (
            W::InsertText { at, .. },
            W::InsertText {
                at: a2, text: t2, ..
            },
        ) => {
            if *a2 < *at || (*a2 == *at && side == Side::Later) {
                with_at(at + text_length(t2))
            } else {
                op.clone()
            }
        }
        (W::InsertText { at, .. }, W::DeleteRange { start, end, .. }) => {
            with_at(shift_over_delete(*at, *start, *end))
        }
        (
            W::InsertText { at, .. },
            W::FormatRange {
                start,
                end,
                attribute,
                ..
            },
        ) => {
            if *start < *at && *at < *end {
                let mut out = op.clone();
                if let W::InsertText { formats, .. } = &mut out {
                    formats.retain(|f| !f.same_kind(attribute));
                    formats.push(attribute.clone());
                }
                out
            } else {
                op.clone()
            }
        }
        (W::InsertText { .. }, _) => op.clone(),

        (W::DeleteRange { object, start, end }, W::InsertText { at, text, .. }) => {
            let n = text_length(text);
            if *at <= *start {
                W::DeleteRange {
                    object: *object,
                    start: start + n,
                    end: end + n,
                }
            } else if *at < *end {
                // The text typed inside stays: delete round it, the later part first.
                batch_or_single(vec![
                    W::DeleteRange {
                        object: *object,
                        start: at + n,
                        end: end + n,
                    },
                    W::DeleteRange {
                        object: *object,
                        start: *start,
                        end: *at,
                    },
                ])
            } else {
                op.clone()
            }
        }
        (
            W::DeleteRange { object, start, end },
            W::DeleteRange {
                start: s2, end: e2, ..
            },
        ) => {
            let (start, end) = (
                shift_over_delete(*start, *s2, *e2),
                shift_over_delete(*end, *s2, *e2),
            );
            if start >= end {
                W::Noop
            } else {
                W::DeleteRange {
                    object: *object,
                    start,
                    end,
                }
            }
        }
        (W::DeleteRange { .. }, _) => op.clone(),

        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
            },
            W::InsertText { at, text, .. },
        ) => {
            let n = text_length(text);
            if *at <= *start {
                format(*object, start + n, end + n, attribute)
            } else if *at < *end {
                format(*object, *start, end + n, attribute)
            } else {
                op.clone()
            }
        }
        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
            },
            W::DeleteRange {
                start: s2, end: e2, ..
            },
        ) => {
            let (start, end) = (
                shift_over_delete(*start, *s2, *e2),
                shift_over_delete(*end, *s2, *e2),
            );
            if start >= end {
                W::Noop
            } else {
                format(*object, start, end, attribute)
            }
        }
        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
            },
            W::FormatRange {
                start: s2,
                end: e2,
                attribute: a2,
                ..
            },
        ) if attribute.same_kind(a2) && side == Side::Earlier => {
            // The later format of the same attribute wins where they overlap.
            batch_or_single(
                subtract(*start, *end, *s2, *e2)
                    .into_iter()
                    .map(|(s, e)| format(*object, s, e, attribute))
                    .collect(),
            )
        }
        _ => op.clone(),
    }
}
