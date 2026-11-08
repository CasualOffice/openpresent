//! `casual-slide-transaction`: atomic, invertible edit operations on a deck's model.
//!
//! Started from OpenCalc's `casual-calc-transaction` (PROVENANCE.md, owner instruction
//! 42). Its contract is kept: every [`apply`] returns the **inverse** operation, so undo
//! and redo are inverse replay through [`History`], never a second implementation that
//! can drift. The operations are slides' text operations from
//! `docs/24-OPERATIONS-AND-COLLABORATION.md`: insert text, delete a range, format a
//! range, each over one shape's text body.
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
//! The inverse of each operation is [`TextOp::Restore`]: the paragraphs it touched, as
//! they were. So an undo gives back the very model that was read, run for run, and the
//! writer, which patches only what differs from the file (ADR-013), then changes nothing.
//!
//! Nothing here reads a file; offsets come from the host and are checked, never trusted.

use casual_slide_model::shape::ObjectId;
use casual_slide_model::slide::SlideParts;
use casual_slide_model::text::{
    CharacterProperties, LineBreak, Paragraph, TextBody, TextItem, TextRun, UnderlineType,
};

/// An error applying an operation. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TxnError {
    /// No shape has this id, or its part was not read.
    NoSuchObject(ObjectId),
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
}

impl TxnError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSuchObject(_) => "CS-TXN-0001",
            Self::NoText(_) => "CS-TXN-0002",
            Self::OutOfRange { .. } => "CS-TXN-0003",
            Self::EmptyBody(_) => "CS-TXN-0004",
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
}

impl Attribute {
    fn set(&self, p: &mut CharacterProperties) {
        match self {
            Self::Bold(v) => p.bold = *v,
            Self::Italic(v) => p.italic = *v,
            Self::Underline(v) => p.underline = *v,
        }
    }
}

/// One text operation (`docs/24`, the Text family).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextOp {
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
    /// Several operations as one: applied in order, all or nothing, one undo step.
    Batch(Vec<TextOp>),
}

impl TextOp {
    /// The shape the operation edits; for a batch, its first member's.
    pub fn object(&self) -> Option<ObjectId> {
        match self {
            Self::InsertText { object, .. }
            | Self::DeleteRange { object, .. }
            | Self::FormatRange { object, .. }
            | Self::Restore { object, .. } => Some(*object),
            Self::Batch(ops) => ops.iter().find_map(TextOp::object),
        }
    }

    /// A short name for a menu label: "Undo Typing".
    pub fn label(&self) -> &'static str {
        match self {
            Self::InsertText { .. } => "Typing",
            Self::DeleteRange { .. } => "Delete",
            Self::FormatRange { .. } => "Format",
            Self::Restore { .. } => "Edit",
            Self::Batch(ops) => ops.first().map_or("Edit", TextOp::label),
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

fn properties_of(item: &TextItem) -> Option<&CharacterProperties> {
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

fn insert_break(p: &mut Paragraph, c: usize) {
    let properties = typing_properties(p, c);
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
fn keep_char(ch: char) -> bool {
    !(ch.is_control() && ch != '\t' || matches!(ch, '\u{FFFE}' | '\u{FFFF}'))
}

/// A paragraph and an item index in it.
type Place = (usize, usize);

/// What applying did: the inverse, and whether anything changed.
type Applied = (TextOp, bool);

fn apply_insert(
    body: &mut TextBody,
    object: ObjectId,
    at: usize,
    text: &str,
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
            insert_plain(p, *c, pending);
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
                    insert_break(p, c);
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
        TextOp::Restore {
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
            TextOp::Restore {
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
        TextOp::Restore {
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
        TextOp::Restore {
            object,
            first: p0,
            count: before.len(),
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
        TextOp::Restore {
            object,
            first,
            count: new_count,
            paragraphs: old,
        },
        changed,
    ))
}

fn apply_inner(parts: &mut SlideParts, op: TextOp) -> Result<Applied, TxnError> {
    match op {
        TextOp::InsertText { object, at, text } => {
            apply_insert(body_mut(parts, object)?, object, at, &text)
        }
        TextOp::DeleteRange { object, start, end } => {
            apply_delete(body_mut(parts, object)?, object, start, end)
        }
        TextOp::FormatRange {
            object,
            start,
            end,
            attribute,
        } => apply_format(body_mut(parts, object)?, object, start, end, &attribute),
        TextOp::Restore {
            object,
            first,
            count,
            paragraphs,
        } => apply_restore(body_mut(parts, object)?, object, first, count, paragraphs),
        TextOp::Batch(ops) => {
            let mut inverses: Vec<TextOp> = Vec::with_capacity(ops.len());
            let mut changed = false;
            for op in ops {
                match apply_inner(parts, op) {
                    Ok((inverse, c)) => {
                        changed |= c;
                        inverses.push(inverse);
                    }
                    Err(e) => {
                        // All or nothing: take back what this batch already did.
                        for inverse in inverses.into_iter().rev() {
                            let _ = apply_inner(parts, inverse);
                        }
                        return Err(e);
                    }
                }
            }
            inverses.reverse();
            Ok((TextOp::Batch(inverses), changed))
        }
    }
}

/// Apply `op` to the deck's parts and return its inverse. On an error nothing changed.
pub fn apply(parts: &mut SlideParts, op: TextOp) -> Result<TextOp, TxnError> {
    apply_inner(parts, op).map(|(inverse, _)| inverse)
}

/// Paired undo and redo stacks over [`apply`]. The host keeps one per open deck.
///
/// Copied from OpenCalc's `History`, with its rules: an operation that changed nothing
/// leaves no undo entry and keeps the redo stack; [`History::edits_applied`] counts up and
/// never down, so an undo after a save still reads as unsaved work.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<TextOp>,
    redo: Vec<TextOp>,
    depth: Option<usize>,
    applied: u64,
    pushes: u64,
    sealed: u64,
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
    pub fn apply(&mut self, parts: &mut SlideParts, op: TextOp) -> Result<bool, TxnError> {
        let (inverse, changed) = apply_inner(parts, op)?;
        if !changed {
            return Ok(false);
        }
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
        self.undo.push(TextOp::Batch(members));
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
    pub fn peek_undo(&self) -> Option<&TextOp> {
        self.undo.last()
    }

    /// Undo the last step; returns what it applied, `None` with nothing to undo.
    pub fn undo(&mut self, parts: &mut SlideParts) -> Result<Option<TextOp>, TxnError> {
        let Some(op) = self.undo.pop() else {
            return Ok(None);
        };
        let applied = op.clone();
        let inverse = match apply(parts, op) {
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
    pub fn redo(&mut self, parts: &mut SlideParts) -> Result<Option<TextOp>, TxnError> {
        let Some(op) = self.redo.pop() else {
            return Ok(None);
        };
        let applied = op.clone();
        let inverse = match apply(parts, op) {
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

#[cfg(test)]
mod tests;
