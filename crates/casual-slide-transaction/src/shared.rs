//! A shared session's forms of an edit (`COL-01`, `COL-04`): what is sent, and what
//! undo takes back, each an operation another editor can apply the same way.
//!
//! Alone, typed text takes its format from a neighbour and every inverse is a
//! [`Operation::Restore`] of the paragraphs as they were. Neither travels: a
//! neighbour can differ on another copy by the time the text arrives, and a restore
//! would overwrite what others typed since. So in a shared session typed text is sent
//! as [`Operation::InsertRun`]s carrying the run properties it took, and the inverse
//! kept for undo is the edit that takes it back: a delete for typing, the deleted runs
//! typed again for a delete, and the old values for a format. Those are rebased over
//! what others do, as OpenCalc's collaborative undo is (its docs/69).

use casual_slide_model::slide::SlideParts;
use casual_slide_model::text::{CharacterProperties, TextBody, TextItem};

use crate::collab::wire::text_length;
use crate::{Attribute, Document, Operation, TxnError, apply_inner, properties_of};

/// One stretch of text with one set of run properties. A paragraph break in it has
/// none of its own.
type Piece = (String, Option<Box<CharacterProperties>>);

/// The text of `[start, end)` of a body, cut where run properties change: runs and
/// fields give their characters, a line break a vertical tab, a paragraph end a line
/// feed; an equation or a retained element gives nothing.
pub(crate) fn pieces(body: &TextBody, start: usize, end: usize) -> Vec<Piece> {
    // Each character with its properties, `None` for a paragraph break.
    let mut chars: Vec<(char, Option<Option<&CharacterProperties>>)> = Vec::new();
    let mut at = 0usize;
    for (index, p) in body.paragraphs.iter().enumerate() {
        if index > 0 {
            if (start..end).contains(&at) {
                chars.push(('\n', None));
            }
            at = at.saturating_add(1);
        }
        for item in &p.items {
            let props = properties_of(item);
            let text: Vec<char> = match item {
                TextItem::Run(r) => r.text.as_deref().unwrap_or("").chars().collect(),
                TextItem::Field(f) => f.text.as_deref().unwrap_or("").chars().collect(),
                TextItem::Break(_) => vec!['\u{b}'],
                TextItem::Math(_) | TextItem::Retained(_) => Vec::new(),
            };
            for c in text {
                if (start..end).contains(&at) {
                    chars.push((c, Some(props)));
                }
                at = at.saturating_add(1);
            }
        }
        if at >= end {
            break;
        }
    }
    let mut out: Vec<(String, Option<Option<&CharacterProperties>>)> = Vec::new();
    for (c, props) in chars {
        match (out.last_mut(), props) {
            (Some((text, _)), None) => text.push(c),
            (Some((text, slot @ None)), Some(p)) => {
                text.push(c);
                *slot = Some(p);
            }
            (Some((text, Some(have))), Some(p)) if *have == p => text.push(c),
            (_, props) => out.push((c.to_string(), props)),
        }
    }
    out.into_iter()
        .map(|(text, props)| (text, props.flatten().cloned().map(Box::new)))
        .collect()
}

fn body(parts: &SlideParts, object: crate::ObjectId) -> Option<&TextBody> {
    parts.get(object.part)?.node(object)?.text_body.as_deref()
}

/// `pieces` typed at `at`, one after another.
fn typed(object: crate::ObjectId, at: usize, pieces: Vec<Piece>) -> Operation {
    let mut at = at;
    let mut ops: Vec<Operation> = pieces
        .into_iter()
        .map(|(text, properties)| {
            let here = at;
            at = at.saturating_add(text_length(&text));
            Operation::InsertRun {
                object,
                at: here,
                text,
                properties,
            }
        })
        .collect();
    if ops.len() == 1 {
        ops.remove(0)
    } else {
        Operation::Batch(ops)
    }
}

impl Attribute {
    /// The same attribute with the value `p` has, so setting it gives `p`'s back.
    fn read(&self, p: Option<&CharacterProperties>) -> Self {
        match self {
            Self::Bold(_) => Self::Bold(p.and_then(|p| p.bold)),
            Self::Italic(_) => Self::Italic(p.and_then(|p| p.italic)),
            Self::Underline(_) => Self::Underline(p.and_then(|p| p.underline)),
            Self::Size(_) => Self::Size(p.and_then(|p| p.size)),
            Self::Latin(_) => Self::Latin(p.and_then(|p| p.latin.clone())),
            Self::Fill(_) => Self::Fill(p.and_then(|p| p.fill.clone())),
            Self::Highlight(_) => Self::Highlight(p.and_then(|p| p.highlight.clone().flatten())),
        }
    }
}

/// What takes back `op`, read from the document before it is applied, when it is a
/// text edit; `None` for an edit whose own inverse already travels.
fn undo_form(parts: &SlideParts, op: &Operation) -> Option<Operation> {
    match op {
        Operation::InsertText { object, at, text }
        | Operation::InsertRun {
            object, at, text, ..
        } => Some(Operation::DeleteRange {
            object: *object,
            start: *at,
            end: at.saturating_add(text_length(text)),
        }),
        Operation::DeleteRange { object, start, end } => {
            let body = body(parts, *object)?;
            Some(typed(*object, *start, pieces(body, *start, *end)))
        }
        Operation::FormatRange {
            object,
            start,
            end,
            attribute,
        } => {
            let body = body(parts, *object)?;
            let mut ops: Vec<Operation> = Vec::new();
            let mut at = *start;
            for (text, props) in pieces(body, *start, *end) {
                let n = text_length(&text);
                let old = attribute.read(props.as_deref());
                match ops.last_mut() {
                    Some(Operation::FormatRange {
                        end, attribute: a, ..
                    }) if *a == old && *end == at => *end = at.saturating_add(n),
                    _ => ops.push(Operation::FormatRange {
                        object: *object,
                        start: at,
                        end: at.saturating_add(n),
                        attribute: old,
                    }),
                }
                at = at.saturating_add(n);
            }
            Some(Operation::Batch(ops))
        }
        _ => None,
    }
}

/// What is sent for `op`, read from the document after it was applied: typed text as
/// the runs it became.
fn send_form(parts: &SlideParts, op: Operation) -> Operation {
    match op {
        Operation::InsertText { object, at, text } => {
            let n = text_length(&text);
            match body(parts, object) {
                Some(b) => typed(object, at, pieces(b, at, at.saturating_add(n))),
                None => Operation::InsertText { object, at, text },
            }
        }
        other => other,
    }
}

/// Apply `op` in a shared session: the form to send, the form undo keeps, the inverse,
/// and whether anything changed. A batch is applied member by member, all or nothing,
/// so each member's forms are read in the state it meets.
pub(crate) fn apply_shared<D: Document + ?Sized>(
    doc: &mut D,
    op: Operation,
) -> Result<(Operation, Operation, Operation, bool), TxnError> {
    if let Operation::Batch(ops) = op {
        let mut sent = Vec::with_capacity(ops.len());
        let mut undos = Vec::with_capacity(ops.len());
        let mut inverses = Vec::with_capacity(ops.len());
        let mut changed = false;
        for member in ops {
            match apply_shared(doc, member) {
                Ok((s, u, i, c)) => {
                    sent.push(s);
                    undos.push(u);
                    inverses.push(i);
                    changed |= c;
                }
                Err(e) => {
                    for inverse in inverses.into_iter().rev() {
                        let _ = apply_inner(doc, inverse);
                    }
                    return Err(e);
                }
            }
        }
        undos.reverse();
        inverses.reverse();
        return Ok((
            Operation::Batch(sent),
            Operation::Batch(undos),
            Operation::Batch(inverses),
            changed,
        ));
    }
    let undo = undo_form(doc.parts_mut(), &op);
    let (inverse, changed) = apply_inner(doc, op.clone())?;
    let sent = send_form(doc.parts_mut(), op);
    let undo = undo.unwrap_or_else(|| inverse.clone());
    Ok((sent, undo, inverse, changed))
}

/// The form `op`, already applied to `parts`, is sent in: typed text as the runs it
/// became. For a host or test that applies without a [`crate::History`].
pub fn sent_form(parts: &SlideParts, op: Operation) -> Operation {
    match op {
        Operation::Batch(ops) => {
            Operation::Batch(ops.into_iter().map(|o| sent_form(parts, o)).collect())
        }
        other => send_form(parts, other),
    }
}
