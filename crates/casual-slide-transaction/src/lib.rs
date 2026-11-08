//! `casual-slide-transaction`: atomic, invertible edit operations on a deck's model.
//!
//! Started from OpenCalc's `casual-calc-transaction` (PROVENANCE.md, owner instruction
//! 42). Its contract is kept: every [`apply`] returns the **inverse** operation, so undo
//! and redo are inverse replay through [`History`], never a second implementation that
//! can drift. The operations are slides' text operations from
//! `docs/24-OPERATIONS-AND-COLLABORATION.md`: from the Text family, insert text, delete
//! a range and format a range over one shape's text body (`VS-05`); from the Properties
//! and Tree families, set a node's transform and delete or insert a node (`VS-06`, moved
//! here from `casual-slide-wasm`'s `txn.rs`), and set a shape's look, its geometry,
//! adjust values, fill, line and style (`SHAPE-01`); from the Deck family, add, list,
//! remove, move and hide slides (`VS-08`); from the Motion family, a slide's transition
//! and the effects of its main animation sequence (`VS-10`, [`animate`]). One op set, one
//! undo stack.
//!
//! # Positions
//!
//! A position is `(object, offset)`: the shape by its [`ObjectId`], and a character
//! offset into its body counted in Unicode scalar values. A paragraph break counts as one
//! character, so splitting and merging paragraphs are inserts and deletes of a break. In a
//! paragraph a run counts its text, a line break (`a:br`) one, a field its shown text (and
//! is never split), and an equation or a retained element nothing.
//!
//! # Inverses
//!
//! The inverse of each operation is [`Operation::Restore`]: the paragraphs it touched, as
//! they were. So an undo gives back the very model that was read, run for run, and the
//! writer, which patches only what differs from the file (ADR-013), then changes nothing.
//!
//! Nothing here reads a file; offsets come from the host and are checked, never trusted.

use casual_slide_model::color::Color;
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::presentation::TextFont;
use casual_slide_model::shape::{ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::{Background, PartEntry, SlidePart, SlideParts};
use casual_slide_model::text::{
    Bullet, BulletFont, CharacterProperties, LineBreak, Paragraph, ParagraphProperties, TextAlign,
    TextBody, TextItem, TextRun, TextSpacing, UnderlineType,
};
use casual_slide_model::theme::{ColorScheme, FontScheme, ShapeStyle};
use casual_slide_model::timing::{SlideTiming, Slot};
use casual_slide_model::transition::SlideTransition;

pub mod collab;
mod design;
pub use design::{Document, MAX_SLIDE_SIZE, MIN_SLIDE_SIZE};
pub mod rich;
mod shared;
pub use shared::sent_form;

/// An error applying an operation. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TxnError {
    /// No shape has this id, or its part was not read.
    NoSuchObject(ObjectId),
    /// The shape tree's own root cannot be removed, nor a node put under a parent
    /// that is not there.
    NoParent(ObjectId),
    /// The shape has no text body to edit.
    NoText(ObjectId),
    /// An offset or a paragraph range is past the end of the body.
    OutOfRange {
        /// The offset asked for.
        offset: usize,
        /// The body's length.
        length: usize,
    },
    /// A restore would leave a body with no paragraph, which the schema forbids.
    EmptyBody(ObjectId),
    /// A design operation reached a document that holds no themes or slide size, or a
    /// master whose theme was not read (`VS-12`).
    NoDesign,
    /// A slide size outside `ST_SlideSizeCoordinate`, 914400 to 51206400 EMU.
    BadSlideSize {
        /// The width asked for.
        cx: i64,
        /// The height asked for.
        cy: i64,
    },
    /// No slide is at this place in the deck, or a slide cannot go there (`VS-08`).
    NoSuchSlide(usize),
    /// A part cannot be added or dropped here: an added part must take the next
    /// ordinal, and only the last part, listed nowhere, can be dropped (`VS-08`).
    BadPart(u32),
    /// The slide's timing is not in the shape the Animations tab edits: inside
    /// `mc:AlternateContent`, left unread, or a root holding something else. Nothing
    /// changes, so nothing the edit cannot read is lost (`VS-10`).
    Timing(u32),
    /// No shape on the slide has this `cNvPr/@id`, so no effect can name it (`VS-10`).
    NoTarget(u32),
    /// The node holds no table (`VS-11`).
    NoTable(ObjectId),
    /// A row or column edit the table cannot take: a place past its end, its last row
    /// or column, more than [`rich::MAX_TABLE_SIDE`], or merged cells (`VS-11`).
    TableShape,
    /// No chart part of this name is in the document (`VS-11`).
    NoChart(String),
    /// A shared session cannot send this edit yet, so it was not applied (`VS-09`).
    NotShared,
}

impl TxnError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSuchObject(_) => "CS-TXN-0001",
            Self::NoParent(_) => "CS-TXN-0002",
            Self::NoText(_) => "CS-TXN-0003",
            Self::OutOfRange { .. } => "CS-TXN-0004",
            Self::EmptyBody(_) => "CS-TXN-0005",
            Self::NoDesign => "CS-TXN-0010",
            Self::BadSlideSize { .. } => "CS-TXN-0011",
            Self::NoSuchSlide(_) => "CS-TXN-0006",
            Self::BadPart(_) => "CS-TXN-0007",
            Self::Timing(_) => "CS-TXN-0008",
            Self::NoTarget(_) => "CS-TXN-0009",
            Self::NoTable(_) => "CS-TXN-0012",
            Self::TableShape => "CS-TXN-0013",
            Self::NoChart(_) => "CS-TXN-0014",
            Self::NotShared => "CS-TXN-0015",
        }
    }
}

impl core::fmt::Display for TxnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchObject(id) => write!(
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
            Self::NoText(id) => write!(
                f,
                "[{}] shape {} in part {} has no text body",
                self.code(),
                id.node,
                id.part
            ),
            Self::OutOfRange { offset, length } => write!(
                f,
                "[{}] offset {offset} is past the body's {length} characters",
                self.code()
            ),
            Self::EmptyBody(id) => write!(
                f,
                "[{}] shape {} in part {} would be left with no paragraph",
                self.code(),
                id.node,
                id.part
            ),
            Self::NoDesign => write!(
                f,
                "[{}] no theme or slide size to change in this document",
                self.code()
            ),
            Self::BadSlideSize { cx, cy } => write!(
                f,
                "[{}] a slide of {cx} by {cy} EMU is outside 914400 to 51206400",
                self.code()
            ),
            Self::NoSuchSlide(index) => {
                write!(f, "[{}] no slide at place {index}", self.code())
            }
            Self::BadPart(ordinal) => write!(
                f,
                "[{}] part {ordinal} cannot be added or dropped there",
                self.code()
            ),
            Self::Timing(ordinal) => write!(
                f,
                "[{}] the animations of part {ordinal} are not in a form this build edits yet",
                self.code()
            ),
            Self::NoTarget(spid) => {
                write!(f, "[{}] no shape on the slide has id {spid}", self.code())
            }
            Self::NoTable(id) => write!(
                f,
                "[{}] shape {} in part {} holds no table",
                self.code(),
                id.node,
                id.part
            ),
            Self::TableShape => write!(
                f,
                "[{}] the table cannot take that row or column edit",
                self.code()
            ),
            Self::NoChart(_) => write!(f, "[{}] no such chart part", self.code()),
            Self::NotShared => write!(
                f,
                "[{}] this edit cannot be shared with other editors yet",
                self.code()
            ),
        }
    }
}

impl std::error::Error for TxnError {}

/// A character attribute a range can be given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    /// `rPr/@b`; `None` removes it, so the run inherits again.
    Bold(Option<bool>),
    /// `rPr/@i`.
    Italic(Option<bool>),
    /// `rPr/@u`.
    Underline(Option<UnderlineType>),
    /// `rPr/@sz`, hundredths of a point: the Font Size box, Increase and Decrease.
    Size(Option<i32>),
    /// `rPr/latin`: the Font box. Only the Latin font, as PowerPoint's box sets.
    Latin(Option<Box<TextFont>>),
    /// The run's `EG_FillProperties` choice: Font Color.
    Fill(Option<Fill>),
    /// `rPr/highlight`: Text Highlight Color. `None` removes it.
    Highlight(Option<Color>),
}

impl Attribute {
    fn set(&self, p: &mut CharacterProperties) {
        match self {
            Self::Bold(v) => p.bold = *v,
            Self::Italic(v) => p.italic = *v,
            Self::Underline(v) => p.underline = *v,
            Self::Size(v) => p.size = *v,
            Self::Latin(v) => p.latin.clone_from(v),
            Self::Fill(v) => p.fill.clone_from(v),
            Self::Highlight(v) => p.highlight = v.clone().map(Some),
        }
    }
}

/// A paragraph property the Home tab's Paragraph group sets.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParagraphChange {
    /// `pPr/@algn`. `None` removes it, so the paragraph inherits again.
    Align(Option<TextAlign>),
    /// The `EG_TextBullet` choice and the `EG_TextBulletTypeface` choice with it.
    Bullet(Option<Bullet>, Option<BulletFont>),
    /// `pPr/@marL` and `pPr/@indent`, EMU: where a bullet hangs.
    Indent {
        /// `@marL`.
        margin_left: Option<i32>,
        /// `@indent`.
        indent: Option<i32>,
    },
    /// `pPr/@lvl`, from 0: Increase and Decrease List Level, Tab and Shift+Tab.
    Level(Option<u8>),
    /// `pPr/@marL` alone, EMU: where a list level's text starts when the paragraph
    /// states its own margin.
    MarginLeft(Option<i32>),
    /// `pPr/lnSpc`: Line Spacing.
    LineSpacing(Option<TextSpacing>),
}

impl ParagraphChange {
    fn set(&self, p: &mut ParagraphProperties) {
        match self {
            Self::Align(v) => p.align = *v,
            Self::Bullet(b, f) => {
                p.bullet.clone_from(b);
                p.bullet_font.clone_from(f);
            }
            Self::Indent {
                margin_left,
                indent,
            } => {
                p.margin_left = *margin_left;
                p.indent = *indent;
            }
            Self::Level(v) => p.level = *v,
            Self::MarginLeft(v) => p.margin_left = *v,
            Self::LineSpacing(v) => p.line_spacing.clone_from(v),
        }
    }
}

/// One edit of the model (`docs/24`).
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
    /// Properties: give a node this `spPr`, all but its transform, and this `style`:
    /// its geometry and adjust values, fill, line and effects, and its theme style
    /// references (`docs/24`). The inverse is the same operation with what it had.
    SetLook {
        /// The node.
        node: ObjectId,
        /// Its `spPr`, but `xfrm`, which [`Operation::SetTransform`] owns.
        properties: Box<ShapeProperties>,
        /// Its `p:style`. `None` removes it.
        style: Option<Box<ShapeStyle>>,
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
    /// Insert `text` at `at`. A line feed splits the paragraph, keeping its properties on
    /// both halves; a vertical tab (U+000B) is a line break, as PowerPoint's Shift+Enter.
    /// Other control characters, which XML cannot hold, are dropped. Typed text takes the
    /// properties of the character before it, as typing does.
    InsertText {
        /// The shape.
        object: ObjectId,
        /// Where, in characters.
        at: usize,
        /// What.
        text: String,
    },
    /// Insert `text` at `at` as typed text whose every run and line break has exactly
    /// `properties`, rather than taking them from a neighbour: typed text as it crosses
    /// to another editor, which must format it as its sender did (`COL-01`), and the
    /// undo of a delete in a shared session. A line feed splits the paragraph as in
    /// [`Operation::InsertText`].
    InsertRun {
        /// The shape.
        object: ObjectId,
        /// Where, in characters.
        at: usize,
        /// What.
        text: String,
        /// The run properties it has; `None` writes no `rPr`.
        properties: Option<Box<CharacterProperties>>,
    },
    /// Delete `[start, end)`. Paragraphs it spans merge into the first, which keeps its
    /// own paragraph properties. Elements this model does not know stay where they were.
    DeleteRange {
        /// The shape.
        object: ObjectId,
        /// First character deleted.
        start: usize,
        /// First character kept.
        end: usize,
    },
    /// Give `[start, end)` an attribute, splitting runs at its ends.
    FormatRange {
        /// The shape.
        object: ObjectId,
        /// First character formatted.
        start: usize,
        /// First character not.
        end: usize,
        /// What to set.
        attribute: Attribute,
    },
    /// Give paragraphs `[first, first + count)` a paragraph property. The inverse is a
    /// [`Operation::Restore`] of the paragraphs as they were.
    FormatParagraphs {
        /// The shape.
        object: ObjectId,
        /// First paragraph changed.
        first: usize,
        /// How many.
        count: usize,
        /// What to set.
        change: ParagraphChange,
    },
    /// Replace paragraphs `[first, first + count)` with `paragraphs`. The inverse of every
    /// other operation: the paragraphs it touched, exactly as they were.
    Restore {
        /// The shape.
        object: ObjectId,
        /// First paragraph replaced.
        first: usize,
        /// How many are replaced.
        count: usize,
        /// What goes in their place.
        paragraphs: Vec<Paragraph>,
    },
    /// Design: make a master's theme colour scheme this one; `None` makes it absent
    /// (`VS-12`). Its own inverse, with the scheme it replaced.
    SetColorScheme {
        /// The master's part ordinal.
        master: u32,
        /// What `clrScheme` becomes.
        scheme: Option<Box<ColorScheme>>,
    },
    /// Design: make a master's theme font scheme this one.
    SetFontScheme {
        /// The master's part ordinal.
        master: u32,
        /// What `fontScheme` becomes.
        scheme: Option<Box<FontScheme>>,
    },
    /// Design: the slide size, `sldSz`, in EMU. Content is scaled by `SetTransform`s
    /// batched with it, so this changes the size alone.
    SetSlideSize {
        /// Width.
        cx: i64,
        /// Height.
        cy: i64,
    },
    /// Design: a part's `cSld/bg`; `None` removes it, so the part inherits again.
    SetBackground {
        /// The slide, layout or master.
        part: u32,
        /// What `bg` becomes.
        background: Option<Box<Background>>,
    },
    /// Deck: add a slide-like part with the next ordinal, listed nowhere yet. A new or
    /// duplicated slide is this, then [`Operation::InsertSlide`] (`VS-08`).
    AddPart {
        /// Where it is; `entry.id.part` must be the next ordinal.
        entry: Box<PartEntry>,
        /// What it holds; its `id` is `entry.id`.
        part: Box<SlidePart>,
    },
    /// Deck: drop the last part, which nothing lists. The inverse of
    /// [`Operation::AddPart`].
    DropPart {
        /// Its ordinal.
        ordinal: u32,
    },
    /// Deck: list part `ordinal` as a slide at `index` of `sldIdLst`.
    InsertSlide {
        /// Its place, from 0.
        index: usize,
        /// The slide part.
        ordinal: u32,
    },
    /// Deck: take the slide at `index` out of the deck. Its part stays, so undo can
    /// list it again; a save deletes it from the file.
    RemoveSlide {
        /// Its place, from 0.
        index: usize,
    },
    /// Deck: move the slide at `from` to `to`, counted after it is taken out.
    MoveSlide {
        /// Where it is.
        from: usize,
        /// Where it goes.
        to: usize,
    },
    /// Deck: a slide's `@show`; `Some(false)` hides it in a slide show, `None` removes
    /// the attribute, so absent stays absent.
    SetSlideShow {
        /// The slide part.
        ordinal: u32,
        /// What `@show` becomes.
        show: Option<bool>,
    },
    /// Motion: a slide's `p:transition`; `None` removes it (`VS-10`).
    SetTransition {
        /// The slide part.
        ordinal: u32,
        /// What it becomes.
        transition: Option<Slot<SlideTransition>>,
    },
    /// Motion: a slide's whole `p:timing`; `None` removes it. The inverse of every
    /// effect operation: the timing exactly as it was (`VS-10`).
    SetTiming {
        /// The slide part.
        ordinal: u32,
        /// What it becomes.
        timing: Option<Slot<SlideTiming>>,
    },
    /// Motion: add an effect to the slide's main sequence at `index` (`VS-10`).
    AddEffect {
        /// The slide part.
        ordinal: u32,
        /// Its place in the sequence, from 0.
        index: usize,
        /// What it is.
        effect: NewEffect,
    },
    /// Motion: delete the effect at `index` of the main sequence.
    RemoveEffect {
        /// The slide part.
        ordinal: u32,
        /// Its place.
        index: usize,
    },
    /// Motion: move the effect at `from` to `to`, counted after it is taken out.
    MoveEffect {
        /// The slide part.
        ordinal: u32,
        /// Where it is.
        from: usize,
        /// Where it goes.
        to: usize,
    },
    /// Motion: when the effect at `index` starts.
    SetEffectStart {
        /// The slide part.
        ordinal: u32,
        /// Its place.
        index: usize,
        /// On click, with or after the previous.
        start: EffectStart,
    },
    /// Properties: a graphic frame's table, whole (`VS-11`). Its inverse is the table
    /// it replaced. Refused on a node that holds no table.
    SetTable {
        /// The frame.
        node: ObjectId,
        /// What its table becomes.
        table: Option<Box<casual_slide_model::table::Table>>,
    },
    /// Chart: a chart part's model, whole (`VS-11`): edited data, or a new chart's.
    /// Its inverse is the part it replaced.
    SetChart {
        /// The chart part's name.
        part: String,
        /// What it becomes.
        chart: Box<casual_slide_model::chart::ChartPart>,
    },
    /// Several operations as one: applied in order, all or nothing, one undo step.
    Batch(Vec<Operation>),
}

impl Operation {
    /// The shape the operation edits; for a batch, its first member's.
    pub fn object(&self) -> Option<ObjectId> {
        match self {
            Self::SetTransform { node, .. }
            | Self::SetLook { node, .. }
            | Self::DeleteNode { node } => Some(*node),
            Self::InsertNode { node, .. } => Some(node.id),
            Self::InsertText { object, .. }
            | Self::InsertRun { object, .. }
            | Self::DeleteRange { object, .. }
            | Self::FormatRange { object, .. }
            | Self::FormatParagraphs { object, .. }
            | Self::Restore { object, .. } => Some(*object),
            Self::SetTable { node, .. } => Some(*node),
            Self::SetChart { .. } => None,
            Self::SetColorScheme { .. }
            | Self::SetFontScheme { .. }
            | Self::SetSlideSize { .. }
            | Self::SetBackground { .. } => None,
            Self::Batch(ops) => ops.iter().find_map(Operation::object),
            Self::AddPart { .. }
            | Self::DropPart { .. }
            | Self::InsertSlide { .. }
            | Self::RemoveSlide { .. }
            | Self::MoveSlide { .. }
            | Self::SetSlideShow { .. }
            | Self::SetTransition { .. }
            | Self::SetTiming { .. }
            | Self::AddEffect { .. }
            | Self::RemoveEffect { .. }
            | Self::MoveEffect { .. }
            | Self::SetEffectStart { .. } => None,
        }
    }

    /// The part this operation edits, for a host that shows it. The first, for a batch.
    pub fn part(&self) -> Option<u32> {
        match self {
            Self::SetBackground { part, .. } => Some(*part),
            Self::SetSlideShow { ordinal, .. }
            | Self::InsertSlide { ordinal, .. }
            | Self::SetTransition { ordinal, .. }
            | Self::SetTiming { ordinal, .. }
            | Self::AddEffect { ordinal, .. }
            | Self::RemoveEffect { ordinal, .. }
            | Self::MoveEffect { ordinal, .. }
            | Self::SetEffectStart { ordinal, .. } => Some(*ordinal),
            Self::Batch(ops) => ops.iter().find_map(Operation::part),
            _ => self.object().map(|o| o.part),
        }
    }

    /// Whether this changes the whole deck's look, a theme or the slide size, so a host
    /// redraws every slide and not only [`Operation::part`]'s.
    pub fn is_deck_wide(&self) -> bool {
        match self {
            Self::SetColorScheme { .. }
            | Self::SetFontScheme { .. }
            | Self::SetSlideSize { .. } => true,
            Self::Batch(ops) => ops.iter().any(Operation::is_deck_wide),
            _ => false,
        }
    }

    /// Whether this changes which slides the deck lists, or their order: a host
    /// redraws its slide list rather than one slide.
    pub fn changes_slide_list(&self) -> bool {
        match self {
            Self::InsertSlide { .. } | Self::RemoveSlide { .. } | Self::MoveSlide { .. } => true,
            Self::Batch(ops) => ops.iter().any(Operation::changes_slide_list),
            _ => false,
        }
    }

    /// A short name for a menu label: "Undo Typing".
    pub fn label(&self) -> &'static str {
        match self {
            Self::SetTransform { .. } => "Move",
            Self::SetLook { .. } => "Format Shape",
            Self::DeleteNode { .. } => "Delete",
            Self::InsertNode { .. } => "Insert",
            Self::InsertText { .. } | Self::InsertRun { .. } => "Typing",
            Self::DeleteRange { .. } => "Delete",
            Self::FormatRange { .. } | Self::FormatParagraphs { .. } => "Format",
            Self::Restore { .. } => "Edit",
            Self::SetTable { .. } => "Table",
            Self::SetChart { .. } => "Chart Data",
            Self::SetColorScheme { .. } | Self::SetFontScheme { .. } => "Theme",
            Self::SetSlideSize { .. } => "Slide Size",
            Self::SetBackground { .. } => "Background",
            Self::AddPart { .. } | Self::InsertSlide { .. } => "New Slide",
            Self::DropPart { .. } | Self::RemoveSlide { .. } => "Delete Slide",
            Self::MoveSlide { .. } => "Move Slide",
            Self::SetSlideShow { .. } => "Hide Slide",
            Self::SetTransition { .. } => "Transition",
            Self::SetTiming { .. } | Self::AddEffect { .. } | Self::SetEffectStart { .. } => {
                "Animation"
            }
            Self::RemoveEffect { .. } => "Remove Animation",
            Self::MoveEffect { .. } => "Reorder Animations",
            Self::Batch(ops) => ops.first().map_or("Edit", Operation::label),
        }
    }
}

/// How many characters an item counts for.
pub fn item_length(item: &TextItem) -> usize {
    match item {
        TextItem::Run(r) => r.text.as_deref().map_or(0, |t| t.chars().count()),
        TextItem::Break(_) => 1,
        TextItem::Field(f) => f.text.as_deref().map_or(0, |t| t.chars().count()),
        TextItem::Math(_) | TextItem::Retained(_) => 0,
    }
}

/// How many characters a paragraph counts for, without its break.
pub fn paragraph_length(p: &Paragraph) -> usize {
    p.items
        .iter()
        .map(item_length)
        .fold(0, usize::saturating_add)
}

/// How many characters a body counts for, a break between each two paragraphs.
pub fn body_length(body: &TextBody) -> usize {
    body.paragraphs
        .iter()
        .map(paragraph_length)
        .fold(0usize, usize::saturating_add)
        .saturating_add(body.paragraphs.len().saturating_sub(1))
}

/// A body offset as `(paragraph, offset in it)`.
pub fn locate(body: &TextBody, offset: usize) -> Result<(usize, usize), TxnError> {
    let mut left = offset;
    for (index, p) in body.paragraphs.iter().enumerate() {
        let len = paragraph_length(p);
        if left <= len {
            return Ok((index, left));
        }
        left = left.saturating_sub(len.saturating_add(1));
    }
    Err(TxnError::OutOfRange {
        offset,
        length: body_length(body),
    })
}

/// The body offset of the start of paragraph `index`.
pub fn paragraph_start(body: &TextBody, index: usize) -> usize {
    body.paragraphs
        .iter()
        .take(index)
        .map(|p| paragraph_length(p).saturating_add(1))
        .fold(0, usize::saturating_add)
}

fn body_mut(parts: &mut SlideParts, object: ObjectId) -> Result<&mut TextBody, TxnError> {
    let node = parts
        .get_mut(object.part)
        .and_then(|p| p.node_mut(object))
        .ok_or(TxnError::NoSuchObject(object))?;
    node.text_body
        .as_deref_mut()
        .ok_or(TxnError::NoText(object))
}

pub(crate) fn properties_of(item: &TextItem) -> Option<&CharacterProperties> {
    match item {
        TextItem::Run(r) => r.properties.as_deref(),
        TextItem::Break(b) => b.properties.as_deref(),
        TextItem::Field(f) => f.properties.as_deref(),
        TextItem::Math(_) | TextItem::Retained(_) => None,
    }
}

fn split_chars(text: &str, at: usize) -> (String, String) {
    let byte = text.char_indices().nth(at).map_or(text.len(), |(b, _)| b);
    let (a, b) = text.split_at(byte);
    (a.to_owned(), b.to_owned())
}

/// Make an item boundary at character `c` of a paragraph, splitting a run if it falls
/// inside one, and return the index of the first item after it. A field is never split:
/// the boundary moves to its end.
fn split_at(items: &mut Vec<TextItem>, c: usize) -> usize {
    let mut acc = 0usize;
    for index in 0..items.len() {
        if acc == c {
            return index;
        }
        let Some(item) = items.get(index) else { break };
        let len = item_length(item);
        let end = acc.saturating_add(len);
        if c < end {
            if let Some(TextItem::Run(run)) = items.get(index) {
                let (left, right) = split_chars(run.text.as_deref().unwrap_or(""), c - acc);
                let mut tail = run.clone();
                tail.text = Some(right);
                if let Some(TextItem::Run(run)) = items.get_mut(index) {
                    run.text = Some(left);
                }
                items.insert(index.saturating_add(1), TextItem::Run(tail));
            }
            return index.saturating_add(1);
        }
        acc = end;
    }
    items.len()
}

/// The properties typing at `c` takes: the character before, else after, else the
/// paragraph's end properties.
fn typing_properties(p: &Paragraph, c: usize) -> Option<Box<CharacterProperties>> {
    let mut acc = 0usize;
    let mut before = None;
    let mut after = None;
    for item in &p.items {
        let len = item_length(item);
        let has = matches!(
            item,
            TextItem::Run(_) | TextItem::Break(_) | TextItem::Field(_)
        );
        if has && acc < c {
            before = Some(properties_of(item).cloned());
        }
        if has && acc >= c && after.is_none() {
            after = Some(properties_of(item).cloned());
        }
        acc = acc.saturating_add(len);
    }
    before
        .or(after)
        .unwrap_or_else(|| p.end_properties.as_deref().cloned())
        .map(Box::new)
}

/// Whether text typed at `offset` of `body` takes the format of what follows rather
/// than of the character before: no run holds the character before, as at a
/// paragraph's start or after a line break or field. Collaboration needs it to grow a
/// concurrent format over typed text exactly as applying them in either order would
/// (`collab::wire`).
pub fn typing_takes_after(body: &TextBody, offset: usize) -> bool {
    let Ok((p, c)) = locate(body, offset) else {
        return false;
    };
    let Some(p) = body.paragraphs.get(p) else {
        return false;
    };
    let mut acc = 0usize;
    for item in &p.items {
        let len = item_length(item);
        if matches!(item, TextItem::Run(_)) && acc < c && c <= acc.saturating_add(len) {
            return false;
        }
        acc = acc.saturating_add(len);
        if acc >= c {
            break;
        }
    }
    true
}

/// Insert plain text, with no break in it, at `c` of a paragraph.
fn insert_plain(p: &mut Paragraph, c: usize, text: &str) {
    if text.is_empty() {
        return;
    }
    // The run holding the character before, else an empty or following run at `c`.
    let mut acc = 0usize;
    let mut target: Option<(usize, usize)> = None;
    for (index, item) in p.items.iter().enumerate() {
        let len = item_length(item);
        if let TextItem::Run(_) = item {
            if acc < c && c <= acc.saturating_add(len) {
                target = Some((index, c - acc));
                break;
            }
            if acc == c && target.is_none() {
                target = Some((index, 0));
            }
        }
        acc = acc.saturating_add(len);
        if acc > c {
            break;
        }
    }
    if let Some((index, at)) = target
        && let Some(TextItem::Run(run)) = p.items.get_mut(index)
    {
        let (left, right) = split_chars(run.text.as_deref().unwrap_or(""), at);
        run.text = Some(format!("{left}{text}{right}"));
        return;
    }
    let properties = typing_properties(p, c);
    let index = split_at(&mut p.items, c);
    p.items.insert(
        index,
        TextItem::Run(TextRun {
            properties,
            text: Some(text.to_owned()),
        }),
    );
}

/// Insert plain text with these run properties, joining a run that has them where
/// one holds the character before or starts at `c`, else as a run of its own.
fn insert_plain_as(
    p: &mut Paragraph,
    c: usize,
    text: &str,
    properties: &Option<Box<CharacterProperties>>,
) {
    if text.is_empty() {
        return;
    }
    let mut acc = 0usize;
    let mut target: Option<(usize, usize)> = None;
    for (index, item) in p.items.iter().enumerate() {
        let len = item_length(item);
        if let TextItem::Run(run) = item
            && run.properties == *properties
        {
            if acc < c && c <= acc.saturating_add(len) {
                target = Some((index, c - acc));
                break;
            }
            if acc == c && target.is_none() {
                target = Some((index, 0));
            }
        }
        acc = acc.saturating_add(len);
        if acc > c {
            break;
        }
    }
    if let Some((index, at)) = target
        && let Some(TextItem::Run(run)) = p.items.get_mut(index)
    {
        let (left, right) = split_chars(run.text.as_deref().unwrap_or(""), at);
        run.text = Some(format!("{left}{text}{right}"));
        return;
    }
    let index = split_at(&mut p.items, c);
    p.items.insert(
        index,
        TextItem::Run(TextRun {
            properties: properties.clone(),
            text: Some(text.to_owned()),
        }),
    );
}

fn insert_break(p: &mut Paragraph, c: usize, given: Option<&Option<Box<CharacterProperties>>>) {
    let properties = match given {
        Some(p) => p.clone(),
        None => typing_properties(p, c),
    };
    let index = split_at(&mut p.items, c);
    p.items
        .insert(index, TextItem::Break(LineBreak { properties }));
}

/// Split a paragraph at `c`; the second half keeps its paragraph and end properties.
fn split_paragraph(p: &mut Paragraph, c: usize) -> Paragraph {
    let index = split_at(&mut p.items, c);
    let tail = p.items.split_off(index);
    Paragraph {
        properties: p.properties.clone(),
        items: tail,
        end_properties: p.end_properties.clone(),
    }
}

/// Whether XML can hold the character, as text a user typed.
pub(crate) fn keep_char(ch: char) -> bool {
    !(ch.is_control() && ch != '\t' || matches!(ch, '\u{FFFE}' | '\u{FFFF}'))
}

/// A paragraph and an item index in it.
type Place = (usize, usize);

/// What applying did: the inverse, and whether anything changed.
type Applied = (Operation, bool);

fn apply_insert(
    body: &mut TextBody,
    object: ObjectId,
    at: usize,
    text: &str,
    given: Option<&Option<Box<CharacterProperties>>>,
) -> Result<Applied, TxnError> {
    let (first, c) = locate(body, at)?;
    let Some(before) = body.paragraphs.get(first).cloned() else {
        return Err(TxnError::OutOfRange {
            offset: at,
            length: body_length(body),
        });
    };
    let mut current = first;
    let mut c = c;
    let mut pending = String::new();
    let flush = |body: &mut TextBody, current: usize, c: &mut usize, pending: &mut String| {
        if let Some(p) = body.paragraphs.get_mut(current) {
            match given {
                Some(properties) => insert_plain_as(p, *c, pending, properties),
                None => insert_plain(p, *c, pending),
            }
            *c = c.saturating_add(pending.chars().count());
        }
        pending.clear();
    };
    for ch in text.chars() {
        match ch {
            '\n' => {
                flush(body, current, &mut c, &mut pending);
                let Some(p) = body.paragraphs.get_mut(current) else {
                    break;
                };
                let tail = split_paragraph(p, c);
                current = current.saturating_add(1);
                body.paragraphs.insert(current, tail);
                c = 0;
            }
            '\u{b}' => {
                flush(body, current, &mut c, &mut pending);
                if let Some(p) = body.paragraphs.get_mut(current) {
                    insert_break(p, c, given);
                    c = c.saturating_add(1);
                }
            }
            ch if keep_char(ch) => pending.push(ch),
            _ => {}
        }
    }
    flush(body, current, &mut c, &mut pending);
    let count = current - first + 1;
    let changed = count != 1 || body.paragraphs.get(first) != Some(&before);
    Ok((
        Operation::Restore {
            object,
            first,
            count,
            paragraphs: vec![before],
        },
        changed,
    ))
}

/// Split both ends of a range into item boundaries; returns the paragraphs and the item
/// index in each where the range starts and ends.
fn bound(body: &mut TextBody, start: usize, end: usize) -> Result<(Place, Place), TxnError> {
    let length = body_length(body);
    if start > end || end > length {
        return Err(TxnError::OutOfRange {
            offset: end.max(start),
            length,
        });
    }
    let (p0, c0) = locate(body, start)?;
    let (p1, c1) = locate(body, end)?;
    let i0 = body
        .paragraphs
        .get_mut(p0)
        .map_or(0, |p| split_at(&mut p.items, c0));
    let i1 = body
        .paragraphs
        .get_mut(p1)
        .map_or(0, |p| split_at(&mut p.items, c1));
    Ok(((p0, i0), (p1, i1)))
}

fn apply_delete(
    body: &mut TextBody,
    object: ObjectId,
    start: usize,
    end: usize,
) -> Result<Applied, TxnError> {
    let length = body_length(body);
    if start > end || end > length {
        return Err(TxnError::OutOfRange {
            offset: end.max(start),
            length,
        });
    }
    let (p0, _) = locate(body, start)?;
    let (p1, _) = locate(body, end)?;
    let before: Vec<Paragraph> = body.paragraphs.get(p0..=p1).unwrap_or_default().to_vec();
    if start == end {
        return Ok((
            Operation::Restore {
                object,
                first: p0,
                count: 1,
                paragraphs: before,
            },
            false,
        ));
    }
    let ((p0, i0), (p1, i1)) = bound(body, start, end)?;
    // Elements this model does not know are kept, where the deleted text was.
    let keep = |item: &TextItem| matches!(item, TextItem::Retained(_));
    if p0 == p1 {
        if let Some(p) = body.paragraphs.get_mut(p0) {
            let removed: Vec<TextItem> = p.items.drain(i0..i1.max(i0)).collect();
            let kept: Vec<TextItem> = removed.into_iter().filter(keep).collect();
            for (n, item) in kept.into_iter().enumerate() {
                p.items.insert(i0.saturating_add(n), item);
            }
        }
    } else {
        let last = body.paragraphs.get(p1).cloned().unwrap_or_default();
        let mut kept: Vec<TextItem> = Vec::new();
        for p in body
            .paragraphs
            .get(p0.saturating_add(1)..p1)
            .unwrap_or_default()
        {
            kept.extend(p.items.iter().filter(|i| keep(i)).cloned());
        }
        kept.extend(last.items.iter().take(i1).filter(|i| keep(i)).cloned());
        if let Some(p) = body.paragraphs.get_mut(p0) {
            let tail: Vec<TextItem> = p.items.split_off(i0.min(p.items.len()));
            p.items.extend(tail.into_iter().filter(keep));
            p.items.extend(kept);
            p.items.extend(last.items.into_iter().skip(i1));
            p.end_properties = last.end_properties;
        }
        body.paragraphs.drain(p0.saturating_add(1)..=p1);
    }
    let changed = body.paragraphs.get(p0) != before.first() || before.len() != 1;
    Ok((
        Operation::Restore {
            object,
            first: p0,
            count: 1,
            paragraphs: before,
        },
        changed,
    ))
}

fn apply_format(
    body: &mut TextBody,
    object: ObjectId,
    start: usize,
    end: usize,
    attribute: &Attribute,
) -> Result<Applied, TxnError> {
    let length = body_length(body);
    if start > end || end > length {
        return Err(TxnError::OutOfRange {
            offset: end.max(start),
            length,
        });
    }
    let (p0, _) = locate(body, start)?;
    let (p1, _) = locate(body, end)?;
    let before: Vec<Paragraph> = body.paragraphs.get(p0..=p1).unwrap_or_default().to_vec();
    if start < end {
        let ((p0, i0), (p1, i1)) = bound(body, start, end)?;
        for index in p0..=p1 {
            let Some(p) = body.paragraphs.get_mut(index) else {
                continue;
            };
            let from = if index == p0 { i0 } else { 0 };
            let to = if index == p1 { i1 } else { p.items.len() };
            for item in p.items.iter_mut().take(to).skip(from) {
                let slot = match item {
                    TextItem::Run(r) => &mut r.properties,
                    TextItem::Break(b) => &mut b.properties,
                    TextItem::Field(f) => &mut f.properties,
                    TextItem::Math(_) | TextItem::Retained(_) => continue,
                };
                attribute.set(slot.get_or_insert_with(Box::default));
            }
        }
    }
    let changed = body.paragraphs.get(p0..=p1) != Some(before.as_slice());
    Ok((
        Operation::Restore {
            object,
            first: p0,
            count: before.len(),
            paragraphs: before,
        },
        changed,
    ))
}

fn apply_paragraphs(
    body: &mut TextBody,
    object: ObjectId,
    first: usize,
    count: usize,
    change: &ParagraphChange,
) -> Result<Applied, TxnError> {
    let len = body.paragraphs.len();
    let end = first
        .checked_add(count)
        .filter(|&e| e <= len && count > 0)
        .ok_or(TxnError::OutOfRange {
            offset: first.saturating_add(count),
            length: len,
        })?;
    let before: Vec<Paragraph> = body.paragraphs.get(first..end).unwrap_or_default().to_vec();
    for p in body.paragraphs.get_mut(first..end).unwrap_or_default() {
        let had = p.properties.is_some();
        let props = p.properties.get_or_insert_with(Box::default);
        change.set(props);
        // Absent stays absent: clearing what was never there writes no empty `pPr`.
        if !had && **props == ParagraphProperties::default() {
            p.properties = None;
        }
    }
    let changed = body.paragraphs.get(first..end) != Some(before.as_slice());
    Ok((
        Operation::Restore {
            object,
            first,
            count,
            paragraphs: before,
        },
        changed,
    ))
}

fn apply_restore(
    body: &mut TextBody,
    object: ObjectId,
    first: usize,
    count: usize,
    paragraphs: Vec<Paragraph>,
) -> Result<Applied, TxnError> {
    let len = body.paragraphs.len();
    let end = first
        .checked_add(count)
        .filter(|&e| e <= len)
        .ok_or(TxnError::OutOfRange {
            offset: first.saturating_add(count),
            length: len,
        })?;
    if len - count + paragraphs.len() == 0 {
        return Err(TxnError::EmptyBody(object));
    }
    let new_count = paragraphs.len();
    let old: Vec<Paragraph> = body.paragraphs.splice(first..end, paragraphs).collect();
    let changed = body.paragraphs.get(first..first + new_count) != Some(old.as_slice());
    Ok((
        Operation::Restore {
            object,
            first,
            count: new_count,
            paragraphs: old,
        },
        changed,
    ))
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

pub(crate) fn apply_inner<D: Document + ?Sized>(
    doc: &mut D,
    op: Operation,
) -> Result<Applied, TxnError> {
    if let Some(applied) = design::apply_design(doc, &op)? {
        return Ok(applied);
    }
    if let Some(applied) = rich::apply_rich(doc, &op)? {
        return Ok(applied);
    }
    let parts = doc.parts_mut();
    match op {
        Operation::SetTransform { node, transform } => {
            let shape = parts
                .get_mut(node.part)
                .and_then(|p| p.node_mut(node))
                .ok_or(TxnError::NoSuchObject(node))?;
            let changed = shape.transform != transform;
            let before = core::mem::replace(&mut shape.transform, transform);
            Ok((
                Operation::SetTransform {
                    node,
                    transform: before,
                },
                changed,
            ))
        }
        Operation::SetLook {
            node,
            properties,
            style,
        } => {
            let shape = parts
                .get_mut(node.part)
                .and_then(|p| p.node_mut(node))
                .ok_or(TxnError::NoSuchObject(node))?;
            let changed = shape.properties != *properties || shape.style != style;
            let before = core::mem::replace(&mut shape.properties, *properties);
            let before_style = core::mem::replace(&mut shape.style, style);
            Ok((
                Operation::SetLook {
                    node,
                    properties: Box::new(before),
                    style: before_style,
                },
                changed,
            ))
        }
        Operation::DeleteNode { node } => {
            let root = root_mut(parts, node.part).ok_or(TxnError::NoSuchObject(node))?;
            if root.id == node {
                return Err(TxnError::NoParent(node));
            }
            let (parent, index) = parent_of(root, node).ok_or(TxnError::NoSuchObject(node))?;
            let removed = parent.children.remove(index);
            Ok((
                Operation::InsertNode {
                    parent: parent.id,
                    index,
                    node: Box::new(removed),
                },
                true,
            ))
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
            Ok((Operation::DeleteNode { node: id }, true))
        }
        Operation::InsertText { object, at, text } => {
            apply_insert(body_mut(parts, object)?, object, at, &text, None)
        }
        Operation::InsertRun {
            object,
            at,
            text,
            properties,
        } => apply_insert(
            body_mut(parts, object)?,
            object,
            at,
            &text,
            Some(&properties),
        ),
        Operation::DeleteRange { object, start, end } => {
            apply_delete(body_mut(parts, object)?, object, start, end)
        }
        Operation::FormatRange {
            object,
            start,
            end,
            attribute,
        } => apply_format(body_mut(parts, object)?, object, start, end, &attribute),
        Operation::FormatParagraphs {
            object,
            first,
            count,
            change,
        } => apply_paragraphs(body_mut(parts, object)?, object, first, count, &change),
        Operation::Restore {
            object,
            first,
            count,
            paragraphs,
        } => apply_restore(body_mut(parts, object)?, object, first, count, paragraphs),
        // Handled by `design::apply_design` above.
        Operation::SetColorScheme { .. }
        | Operation::SetFontScheme { .. }
        | Operation::SetSlideSize { .. }
        | Operation::SetBackground { .. } => Err(TxnError::NoDesign),
        // Handled by `rich::apply_rich` above.
        Operation::SetTable { node, .. } => Err(TxnError::NoTable(node)),
        Operation::SetChart { part, .. } => Err(TxnError::NoChart(part)),
        Operation::AddPart { entry, part } => slides::add_part(parts, *entry, *part),
        Operation::DropPart { ordinal } => slides::drop_part(parts, ordinal),
        Operation::InsertSlide { index, ordinal } => slides::insert_slide(parts, index, ordinal),
        Operation::RemoveSlide { index } => slides::remove_slide(parts, index),
        Operation::MoveSlide { from, to } => slides::move_slide(parts, from, to),
        Operation::SetSlideShow { ordinal, show } => slides::set_show(parts, ordinal, show),
        Operation::SetTransition {
            ordinal,
            transition,
        } => animate::set_transition(parts, ordinal, transition),
        Operation::SetTiming { ordinal, timing } => animate::set_timing(parts, ordinal, timing),
        Operation::AddEffect {
            ordinal,
            index,
            effect,
        } => animate::add_effect(parts, ordinal, index, effect),
        Operation::RemoveEffect { ordinal, index } => animate::remove_effect(parts, ordinal, index),
        Operation::MoveEffect { ordinal, from, to } => {
            animate::move_effect(parts, ordinal, from, to)
        }
        Operation::SetEffectStart {
            ordinal,
            index,
            start,
        } => animate::set_effect_start(parts, ordinal, index, start),
        Operation::Batch(ops) => {
            let mut inverses: Vec<Operation> = Vec::with_capacity(ops.len());
            let mut changed = false;
            for op in ops {
                match apply_inner(doc, op) {
                    Ok((inverse, c)) => {
                        changed |= c;
                        inverses.push(inverse);
                    }
                    Err(e) => {
                        // All or nothing: take back what this batch already did.
                        for inverse in inverses.into_iter().rev() {
                            let _ = apply_inner(doc, inverse);
                        }
                        return Err(e);
                    }
                }
            }
            inverses.reverse();
            Ok((Operation::Batch(inverses), changed))
        }
    }
}

/// Apply `op` to the deck and return its inverse. On an error nothing changed.
pub fn apply<D: Document + ?Sized>(doc: &mut D, op: Operation) -> Result<Operation, TxnError> {
    apply_inner(doc, op).map(|(inverse, _)| inverse)
}

/// Paired undo and redo stacks over [`apply`]. The host keeps one per open deck.
///
/// Copied from OpenCalc's `History`, with its rules: an operation that changed nothing
/// leaves no undo entry and keeps the redo stack; [`History::edits_applied`] counts up and
/// never down, so an undo after a save still reads as unsaved work.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Operation>,
    redo: Vec<Operation>,
    depth: Option<usize>,
    applied: u64,
    pushes: u64,
    sealed: u64,
    /// In a shared session: what may be applied (`VS-09`).
    gate: Option<fn(&Operation) -> bool>,
    /// In a shared session: every operation applied, with its inverse, in order, for
    /// the host to send.
    journal: Option<Vec<(Operation, Operation)>>,
}

impl History {
    /// A new, empty history with no bound on how far back undo reaches.
    pub fn new() -> Self {
        Self::default()
    }

    /// A new, empty history keeping at most `depth` undo entries.
    pub fn with_depth(depth: Option<usize>) -> Self {
        Self {
            depth,
            ..Self::default()
        }
    }

    /// Start or stop a shared session (`VS-09`): while `gate` is set, an operation it
    /// refuses is not applied, by edit, undo or redo, and fails with
    /// [`TxnError::NotShared`]; and every operation applied is journaled with its
    /// inverse for [`History::take_journal`].
    pub fn share(&mut self, gate: Option<fn(&Operation) -> bool>) {
        self.gate = gate;
        self.journal = gate.map(|_| Vec::new());
    }

    /// Whether a shared session is on.
    #[must_use]
    pub fn is_shared(&self) -> bool {
        self.gate.is_some()
    }

    /// Every operation applied since the last call, with its inverse, in order.
    pub fn take_journal(&mut self) -> Vec<(Operation, Operation)> {
        self.journal
            .as_mut()
            .map(core::mem::take)
            .unwrap_or_default()
    }

    fn admit(&self, op: &Operation) -> Result<(), TxnError> {
        match self.gate {
            Some(gate) if !gate(op) => Err(TxnError::NotShared),
            _ => Ok(()),
        }
    }

    fn note(&mut self, op: Operation, inverse: &Operation) {
        if let Some(journal) = self.journal.as_mut() {
            journal.push((op, inverse.clone()));
        }
    }

    fn trim(&mut self) {
        if let Some(depth) = self.depth {
            let excess = self.undo.len().saturating_sub(depth);
            self.undo.drain(..excess);
        }
    }

    fn seal(&mut self) {
        self.pushes = self.pushes.saturating_add(1);
        self.sealed = self.pushes;
    }

    /// Apply `op`, recording its inverse and clearing redo. Returns whether anything
    /// changed; a change of nothing leaves no undo entry.
    pub fn apply<D: Document + ?Sized>(
        &mut self,
        doc: &mut D,
        op: Operation,
    ) -> Result<bool, TxnError> {
        self.admit(&op)?;
        let inverse = if self.journal.is_some() {
            // Shared: send what another copy applies the same way, and keep for undo
            // what takes this back over others' edits (`shared`).
            let (sent, undo, _, changed) = shared::apply_shared(doc, op)?;
            if !changed {
                return Ok(false);
            }
            self.note(sent, &undo);
            undo
        } else {
            let (inverse, changed) = apply_inner(doc, op)?;
            if !changed {
                return Ok(false);
            }
            inverse
        };
        self.applied = self.applied.saturating_add(1);
        self.pushes = self.pushes.saturating_add(1);
        self.undo.push(inverse);
        self.trim();
        self.redo.clear();
        Ok(true)
    }

    /// A mark to pass to [`History::group_since`] once a gesture's edits are applied.
    #[must_use]
    pub fn undo_mark(&self) -> u64 {
        self.pushes
    }

    /// Fold every undo entry pushed since `mark` into one, so one undo takes back the
    /// whole gesture: a burst of typing. Refuses (`false`) if an undo or redo came since.
    pub fn group_since(&mut self, mark: u64) -> bool {
        if self.sealed > mark || mark > self.pushes {
            return false;
        }
        let Ok(n) = usize::try_from(self.pushes - mark) else {
            return false;
        };
        if n > self.undo.len() {
            return false;
        }
        if n < 2 {
            return true;
        }
        // Inverses run newest first.
        let mut members = self.undo.split_off(self.undo.len() - n);
        members.reverse();
        self.undo.push(Operation::Batch(members));
        self.pushes -= n as u64 - 1;
        true
    }

    /// How many edits have ever been applied, undo and redo included.
    pub fn edits_applied(&self) -> u64 {
        self.applied
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// What undo would apply, without applying it.
    pub fn peek_undo(&self) -> Option<&Operation> {
        self.undo.last()
    }

    /// Apply an undo or redo step, journaling it; its inverse, in the shared form when
    /// a session is on.
    fn step<D: Document + ?Sized>(
        &mut self,
        doc: &mut D,
        op: Operation,
    ) -> Result<Operation, TxnError> {
        if self.journal.is_some() {
            let (sent, undo, _, _) = shared::apply_shared(doc, op)?;
            self.note(sent, &undo);
            Ok(undo)
        } else {
            apply(doc, op)
        }
    }

    /// Give `f` the undo stack, then the redo stack, each oldest first, to rebase
    /// over an edit another participant made (`docs/24` §Undo). The top of each is
    /// the last entry.
    pub fn rebase_stacks(&mut self, f: &mut dyn FnMut(&mut [Operation])) {
        f(&mut self.undo);
        f(&mut self.redo);
    }

    /// Undo the last step; returns what it applied, `None` with nothing to undo.
    pub fn undo<D: Document + ?Sized>(
        &mut self,
        doc: &mut D,
    ) -> Result<Option<Operation>, TxnError> {
        let Some(op) = self.undo.pop() else {
            return Ok(None);
        };
        if let Err(e) = self.admit(&op) {
            self.undo.push(op);
            return Err(e);
        }
        let applied = op.clone();
        let inverse = match self.step(doc, op) {
            Ok(i) => i,
            Err(e) => {
                self.undo.push(applied);
                return Err(e);
            }
        };
        self.seal();
        self.applied = self.applied.saturating_add(1);
        self.redo.push(inverse);
        Ok(Some(applied))
    }

    /// Redo the last undone step; returns what it applied.
    pub fn redo<D: Document + ?Sized>(
        &mut self,
        doc: &mut D,
    ) -> Result<Option<Operation>, TxnError> {
        let Some(op) = self.redo.pop() else {
            return Ok(None);
        };
        if let Err(e) = self.admit(&op) {
            self.redo.push(op);
            return Err(e);
        }
        let applied = op.clone();
        let inverse = match self.step(doc, op) {
            Ok(i) => i,
            Err(e) => {
                self.redo.push(applied);
                return Err(e);
            }
        };
        self.seal();
        self.applied = self.applied.saturating_add(1);
        self.undo.push(inverse);
        Ok(Some(applied))
    }
}

pub mod animate;
mod slides;

pub use animate::{EffectKind, EffectStart, NewEffect, TransitionKind};

#[cfg(test)]
mod animate_tests;
#[cfg(test)]
mod rich_tests;
#[cfg(test)]
mod shape_tests;
#[cfg(test)]
mod slide_tests;
#[cfg(test)]
mod tests;
