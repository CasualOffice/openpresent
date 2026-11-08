//! Review operations (`UX-011`): modern comments and their authors, a slide's notes slide,
//! and a shape's non-visual properties, its name and whether it is hidden, as the Selection
//! Pane edits them.
//!
//! Comments and authors are not slide parts, so they reach the deck through
//! [`Document::review_mut`], which a host implements over whatever holds them. Each
//! operation sets a whole value and returns the value it replaced, so undo gives back the
//! very model that was read and the writer then patches nothing.

use std::collections::BTreeMap;

use casual_slide_model::comment::{AuthorList, ThreadedCommentList};
use casual_slide_model::shape::{NonVisualDrawing, ObjectId};
use casual_slide_model::slide::{SlideKind, SlideParts};

use crate::{Applied, Document, Operation, TxnError};

/// A deck's modern comments, by the ordinal of the slide that holds each list, and its
/// modern author list. A slide absent from `comments` has no comment part; one present
/// with an empty list has one that holds nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Review {
    /// Each slide's `p188:cmLst`, by slide ordinal.
    pub comments: BTreeMap<u32, ThreadedCommentList>,
    /// `p188:authorLst`.
    pub authors: Option<AuthorList>,
}

fn review<D: Document + ?Sized>(doc: &mut D) -> Result<&mut Review, TxnError> {
    doc.review_mut().ok_or(TxnError::NoReview)
}

fn is_kind(parts: &SlideParts, ordinal: u32, kind: SlideKind) -> bool {
    parts
        .structure
        .entry(ordinal)
        .is_some_and(|e| e.kind == kind)
}

/// Apply a review operation; `None` for any other, which the caller applies.
pub(crate) fn apply_review<D: Document + ?Sized>(
    doc: &mut D,
    op: &Operation,
) -> Result<Option<Applied>, TxnError> {
    let applied = match op {
        Operation::SetComments { ordinal, comments } => {
            let slide = *ordinal;
            if !is_kind(doc.parts_mut(), slide, SlideKind::Slide) {
                return Err(TxnError::BadPart(slide));
            }
            let map = &mut review(doc)?.comments;
            let before = match comments.as_deref() {
                Some(list) => map.insert(slide, list.clone()),
                None => map.remove(&slide),
            };
            let changed = before.as_ref() != comments.as_deref();
            (
                Operation::SetComments {
                    ordinal: slide,
                    comments: before.map(Box::new),
                },
                changed,
            )
        }
        Operation::SetAuthors { authors } => {
            let slot = &mut review(doc)?.authors;
            let wanted = authors.as_deref().cloned();
            let changed = *slot != wanted;
            let before = core::mem::replace(slot, wanted);
            (
                Operation::SetAuthors {
                    authors: before.map(Box::new),
                },
                changed,
            )
        }
        Operation::SetNotes { ordinal, notes } => set_notes(doc.parts_mut(), *ordinal, *notes)?,
        Operation::SetNonVisual { node, non_visual } => {
            set_non_visual(doc.parts_mut(), *node, non_visual.as_deref())?
        }
        _ => return Ok(None),
    };
    Ok(Some(applied))
}

/// Link slide `slide` to notes slide `notes`, or unlink it. A notes part is linked to one
/// slide at most, so a link to one another slide holds is refused.
fn set_notes(parts: &mut SlideParts, slide: u32, notes: Option<u32>) -> Result<Applied, TxnError> {
    if !is_kind(parts, slide, SlideKind::Slide) {
        return Err(TxnError::BadPart(slide));
    }
    if let Some(n) = notes {
        let taken = parts
            .structure
            .parts
            .iter()
            .any(|e| e.id.part != slide && e.notes == Some(n));
        if !is_kind(parts, n, SlideKind::Notes) || parts.get(n).is_none() || taken {
            return Err(TxnError::BadPart(n));
        }
    }
    let entry = parts
        .structure
        .parts
        .get_mut(usize::try_from(slide).map_err(|_| TxnError::BadPart(slide))?)
        .ok_or(TxnError::BadPart(slide))?;
    let before = core::mem::replace(&mut entry.notes, notes);
    if let Some(old) = before {
        parts.structure.notes.retain(|&o| o != old);
    }
    if let Some(n) = notes
        && !parts.structure.notes.contains(&n)
    {
        parts.structure.notes.push(n);
    }
    Ok((
        Operation::SetNotes {
            ordinal: slide,
            notes: before,
        },
        before != notes,
    ))
}

fn set_non_visual(
    parts: &mut SlideParts,
    node: ObjectId,
    wanted: Option<&NonVisualDrawing>,
) -> Result<Applied, TxnError> {
    let shape = parts
        .get_mut(node.part)
        .and_then(|p| p.node_mut(node))
        .ok_or(TxnError::NoSuchObject(node))?;
    if node.node == 0 {
        return Err(TxnError::NoSuchObject(node));
    }
    // The file's `@id` is the shape's identity in the file: timing and connectors name it.
    if wanted.map(|n| n.spid) != shape.non_visual.as_ref().map(|n| n.spid) {
        return Err(TxnError::NoSuchObject(node));
    }
    let wanted = wanted.cloned();
    let changed = shape.non_visual != wanted;
    let before = core::mem::replace(&mut shape.non_visual, wanted);
    Ok((
        Operation::SetNonVisual {
            node,
            non_visual: before.map(Box::new),
        },
        changed,
    ))
}
