//! Slides' operations on the wire, and their transform (`docs/24` §The op set, §Text OT).
//!
//! The first co-editing slice (`VS-09`) carries the Text family (insert, delete,
//! format), a shape's transform from Properties, and insert and delete of a top-level
//! node from Tree. Every other operation is refused in a shared session before it is
//! applied ([`shareable`]), so two copies never part because one edit could not be
//! sent.
//!
//! Conflict rules, as `docs/24` states them:
//! - **Properties**: per property, the server's order wins: of two concurrent
//!   transforms of one shape, the later stands and the earlier becomes nothing.
//! - **Text**: character OT inside one body. Positions count a paragraph break as one
//!   character. A format over a concurrently deleted range formats what remains; text
//!   inserted inside or at the end of a formatted range is formatted with it, since it
//!   inherits from the character before it as typing does.
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
use casual_slide_model::text::UnderlineType;
use serde::{Deserialize, Serialize};

use super::{Engine, Side};
use crate::{Attribute, Operation};

/// The protocol version of slides' collaboration. 1 is `VS-09`'s first slice.
pub const PROTOCOL_VERSION: u32 = 1;

/// The most characters one inserted text may carry.
pub const MAX_WIRE_TEXT: usize = 1 << 20;

/// The most bytes a node's XML may carry.
pub const MAX_WIRE_XML: usize = 1 << 20;

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

/// A character attribute on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum WireAttribute {
    /// `@b`; `None` removes it.
    Bold(Option<bool>),
    /// `@i`.
    Italic(Option<bool>),
    /// `@u`, as its token.
    Underline(Option<String>),
}

impl WireAttribute {
    fn same_kind(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
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
        /// The element: `p:sp`, `p:cxnSp`, and so on.
        xml: String,
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
    /// Insert text into a shape's body.
    InsertText {
        /// The shape.
        object: Id,
        /// Where, in characters.
        at: usize,
        /// What.
        text: String,
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
        /// Whether text typed at `start` takes the format of the character after it
        /// rather than before (a paragraph's start, or after a line break), so the
        /// range grows over it. Set by the sender, which has the body.
        #[serde(default)]
        start_takes_after: bool,
        /// The same, at `end`.
        #[serde(default)]
        end_takes_after: bool,
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

/// Whether `op` can be shared in this slice, so a shared session may apply it.
#[must_use]
pub fn shareable(op: &Operation) -> bool {
    match op {
        Operation::SetTransform { .. }
        | Operation::DeleteNode { .. }
        | Operation::InsertNode { .. }
        | Operation::InsertText { .. }
        | Operation::DeleteRange { .. }
        | Operation::FormatRange { .. } => true,
        Operation::Batch(ops) => ops.iter().all(shareable),
        _ => false,
    }
}

impl WireOp {
    /// The wire form of `op`, applied locally with `inverse` returned. A delete takes
    /// its parent and place from its inverse; an insert takes its XML from `xml_of`.
    pub fn of(
        op: &Operation,
        inverse: &Operation,
        xml_of: &dyn Fn(ObjectId) -> Option<String>,
        takes_after: &dyn Fn(ObjectId, usize) -> bool,
    ) -> Result<Self, WireError> {
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
                xml: xml_of(node.id).ok_or(WireError::NotShared)?,
            },
            (Operation::DeleteNode { node }, Operation::InsertNode { parent, index, .. }) => {
                Self::DeleteNode {
                    node: (*node).into(),
                    parent: (*parent).into(),
                    index: *index,
                }
            }
            (Operation::InsertText { object, at, text }, _) => Self::InsertText {
                object: (*object).into(),
                at: *at,
                text: text.clone(),
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
                start_takes_after: takes_after(*object, *start),
                end_takes_after: takes_after(*object, *end),
                attribute: match attribute {
                    Attribute::Bold(v) => WireAttribute::Bold(*v),
                    Attribute::Italic(v) => WireAttribute::Italic(*v),
                    Attribute::Underline(v) => {
                        WireAttribute::Underline(v.as_ref().map(Lexical::to_lexical))
                    }
                    // Size, font, colour and highlight are not on the wire yet, so a shared
                    // session refuses them, as it does every edit it cannot carry (`VS-09`).
                    Attribute::Size(_)
                    | Attribute::Latin(_)
                    | Attribute::Fill(_)
                    | Attribute::Highlight(_) => return Err(WireError::NotShared),
                },
            },
            (Operation::Batch(ops), Operation::Batch(inverses)) => {
                // A batch's inverses run newest first.
                let mut out = Vec::with_capacity(ops.len());
                for (op, inverse) in ops.iter().zip(inverses.iter().rev()) {
                    out.push(Self::of(op, inverse, xml_of, takes_after)?);
                }
                Self::Batch { ops: out }
            }
            _ => return Err(WireError::NotShared),
        })
    }

    /// The model operation this wire operation is, reading an inserted node's XML with
    /// `read_node`. Values are checked; text is bounded; nothing is trusted.
    pub fn operation(
        &self,
        read_node: &dyn Fn(&str, ObjectId) -> Option<ShapeNode>,
    ) -> Result<Operation, WireError> {
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
            } => {
                if xml.len() > MAX_WIRE_XML {
                    return Err(WireError::BadValue);
                }
                let id: ObjectId = (*node).into();
                if id.part != parent.part || id.node < 2 {
                    return Err(WireError::BadValue);
                }
                let mut read = read_node(xml, id).ok_or(WireError::BadNode)?;
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
            Self::InsertText { object, at, text } => {
                if text.chars().count() > MAX_WIRE_TEXT {
                    return Err(WireError::BadValue);
                }
                Operation::InsertText {
                    object: (*object).into(),
                    at: *at,
                    text: text.clone(),
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
                ..
            } => {
                if start > end {
                    return Err(WireError::BadValue);
                }
                Operation::FormatRange {
                    object: (*object).into(),
                    start: *start,
                    end: *end,
                    attribute: match attribute {
                        WireAttribute::Bold(v) => Attribute::Bold(*v),
                        WireAttribute::Italic(v) => Attribute::Italic(*v),
                        WireAttribute::Underline(None) => Attribute::Underline(None),
                        WireAttribute::Underline(Some(t)) => Attribute::Underline(Some(
                            UnderlineType::parse_lexical(t).ok_or(WireError::BadValue)?,
                        )),
                    },
                }
            }
            Self::Batch { ops } => Operation::Batch(
                ops.iter()
                    .map(|op| op.operation(read_node))
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

/// Whether text typed at `at` goes with a concurrent delete of `[start, end)`: inside
/// it or at either edge. Typed text takes the format of a neighbour when it is applied,
/// and a delete changes which neighbour that is, so text kept beside or inside a
/// deleted range could be formatted differently on two copies. The delete wins, as a
/// delete of a shape beats a concurrent edit of it (`docs/24`).
fn swallowed(at: usize, start: usize, end: usize) -> bool {
    start < end && start <= at && at <= end
}

/// `x` after `[start, end)` is deleted.
fn shift_over_delete(x: usize, start: usize, end: usize) -> usize {
    x - (x.min(end) - x.min(start))
}

fn delete_or_noop(object: Id, start: usize, end: usize) -> WireOp {
    if start >= end {
        WireOp::Noop
    } else {
        WireOp::DeleteRange { object, start, end }
    }
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
            W::InsertNode {
                parent,
                index,
                node,
                xml,
            },
            W::InsertNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => {
            let index = if *i2 < *index || (*i2 == *index && side == Side::Later) {
                index + 1
            } else {
                *index
            };
            W::InsertNode {
                parent: *parent,
                index,
                node: *node,
                xml: xml.clone(),
            }
        }
        (
            W::InsertNode {
                parent,
                index,
                node,
                xml,
            },
            W::DeleteNode {
                parent: p2,
                index: i2,
                ..
            },
        ) if parent == p2 => W::InsertNode {
            parent: *parent,
            index: if *i2 < *index { index - 1 } else { *index },
            node: *node,
            xml: xml.clone(),
        },
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

fn transform_text(op: &WireOp, over: &WireOp, side: Side) -> WireOp {
    use WireOp as W;
    match (op, over) {
        (
            W::InsertText { object, at, text },
            W::InsertText {
                at: a2, text: t2, ..
            },
        ) => {
            let at = if *a2 < *at || (*a2 == *at && side == Side::Later) {
                at + text_length(t2)
            } else {
                *at
            };
            W::InsertText {
                object: *object,
                at,
                text: text.clone(),
            }
        }
        (W::InsertText { object, at, text }, W::DeleteRange { start, end, .. }) => {
            if swallowed(*at, *start, *end) {
                W::Noop
            } else {
                W::InsertText {
                    object: *object,
                    at: shift_over_delete(*at, *start, *end),
                    text: text.clone(),
                }
            }
        }
        (W::InsertText { .. }, _) => op.clone(),

        (W::DeleteRange { object, start, end }, W::InsertText { at, text, .. }) => {
            let n = text_length(text);
            if swallowed(*at, *start, *end) {
                // The delete beats text typed inside it or at its edges: it takes
                // that text too.
                delete_or_noop(*object, *start, end + n)
            } else if *at < *start {
                W::DeleteRange {
                    object: *object,
                    start: start + n,
                    end: end + n,
                }
            } else {
                op.clone()
            }
        }
        (
            W::DeleteRange { object, start, end },
            W::DeleteRange {
                start: s2, end: e2, ..
            },
        ) => delete_or_noop(
            *object,
            shift_over_delete(*start, *s2, *e2),
            shift_over_delete(*end, *s2, *e2),
        ),
        (W::DeleteRange { .. }, _) => op.clone(),

        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
                start_takes_after,
                end_takes_after,
            },
            W::InsertText { at, text, .. },
        ) => {
            let n = text_length(text);
            // Typed text takes the format of the character before it, or of what
            // follows where no run holds one: at a paragraph's start, and after a
            // break the text itself makes. So the inserted text up to its first break
            // is in the range if the character before it is, and the rest if the
            // character after it is; either order of the two then leaves the same
            // runs.
            let head = text
                .chars()
                .filter(|&c| c == '\n' || c == '\u{b}' || crate::keep_char(c))
                .position(|c| c == '\n' || c == '\u{b}')
                .map_or(n, |i| i + 1);
            let before_in = (*start < *at && *at < *end)
                || (*at == *start && *start_takes_after)
                || (*at == *end && !*end_takes_after);
            let after_in = *start <= *at && *at < *end;
            let mut ranges: Vec<(usize, usize)> = Vec::new();
            let mut add = |s: usize, e: usize| {
                if s >= e {
                    return;
                }
                match ranges.last_mut() {
                    Some(last) if last.1 == s => last.1 = e,
                    _ => ranges.push((s, e)),
                }
            };
            add(*start, (*end).min(*at));
            if before_in {
                add(*at, at + head);
            }
            if after_in {
                add(at + head, at + n);
            }
            add((*start).max(*at) + n, end + n);
            batch_or_single(
                ranges
                    .into_iter()
                    .map(|(start, end)| W::FormatRange {
                        object: *object,
                        start,
                        end,
                        attribute: attribute.clone(),
                        start_takes_after: *start_takes_after,
                        end_takes_after: *end_takes_after,
                    })
                    .collect(),
            )
        }
        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
                start_takes_after,
                end_takes_after,
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
                W::FormatRange {
                    object: *object,
                    start,
                    end,
                    attribute: attribute.clone(),
                    start_takes_after: *start_takes_after,
                    end_takes_after: *end_takes_after,
                }
            }
        }
        (
            W::FormatRange {
                object,
                start,
                end,
                attribute,
                start_takes_after,
                end_takes_after,
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
                    .map(|(start, end)| W::FormatRange {
                        object: *object,
                        start,
                        end,
                        attribute: attribute.clone(),
                        start_takes_after: *start_takes_after,
                        end_takes_after: *end_takes_after,
                    })
                    .collect(),
            )
        }
        _ => op.clone(),
    }
}
