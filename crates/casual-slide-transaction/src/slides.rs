//! The Deck family (`docs/24`, `VS-08`): parts added and dropped, slides listed,
//! taken out, moved and hidden. Each returns its exact inverse, as every operation here
//! does. Nothing reads a file: places and ordinals come from the host and are checked.

use casual_slide_model::slide::{PartEntry, SlideKind, SlidePart, SlideParts};

use super::{Applied, Operation, TxnError};

fn ordinal_of(parts: &SlideParts) -> Option<u32> {
    u32::try_from(parts.parts.len()).ok()
}

/// Append a part with the next ordinal. Refused unless the entry and the part both
/// name that ordinal, so ordinals stay positions.
pub(crate) fn add_part(
    parts: &mut SlideParts,
    entry: PartEntry,
    part: SlidePart,
) -> Result<Applied, TxnError> {
    let next = ordinal_of(parts).ok_or(TxnError::BadPart(entry.id.part))?;
    if parts.structure.parts.len() != parts.parts.len()
        || entry.id.part != next
        || part.id != entry.id
    {
        return Err(TxnError::BadPart(entry.id.part));
    }
    parts.structure.parts.push(entry);
    parts.parts.push(Some(part));
    Ok((Operation::DropPart { ordinal: next }, true))
}

/// Drop the last part. Refused for any other part, for one a slide list or a slide's notes
/// link names, and for anything but a slide or a notes slide, so a layout or master a slide uses is never dropped.
pub(crate) fn drop_part(parts: &mut SlideParts, ordinal: u32) -> Result<Applied, TxnError> {
    let last = ordinal_of(parts)
        .and_then(|n| n.checked_sub(1))
        .ok_or(TxnError::BadPart(ordinal))?;
    let listed = parts.structure.slides.contains(&ordinal)
        || parts
            .structure
            .parts
            .iter()
            .any(|e| e.notes == Some(ordinal));
    // A notes slide no slide links to is dropped as a slide is (`UX-011`).
    let is_slide = parts
        .structure
        .entry(ordinal)
        .is_some_and(|e| matches!(e.kind, SlideKind::Slide | SlideKind::Notes));
    if ordinal != last || listed || !is_slide || parts.structure.parts.len() != parts.parts.len() {
        return Err(TxnError::BadPart(ordinal));
    }
    let entry = parts
        .structure
        .parts
        .pop()
        .ok_or(TxnError::BadPart(ordinal))?;
    let Some(Some(part)) = parts.parts.pop() else {
        // Put the entry back: the part was never read, so it cannot be restored.
        parts.structure.parts.push(entry);
        return Err(TxnError::BadPart(ordinal));
    };
    Ok((
        Operation::AddPart {
            entry: Box::new(entry),
            part: Box::new(part),
        },
        true,
    ))
}

/// List a slide part at `index`.
pub(crate) fn insert_slide(
    parts: &mut SlideParts,
    index: usize,
    ordinal: u32,
) -> Result<Applied, TxnError> {
    let is_slide = parts
        .structure
        .entry(ordinal)
        .is_some_and(|e| e.kind == SlideKind::Slide);
    if !is_slide || parts.get(ordinal).is_none() {
        return Err(TxnError::BadPart(ordinal));
    }
    if index > parts.structure.slides.len() {
        return Err(TxnError::NoSuchSlide(index));
    }
    parts.structure.slides.insert(index, ordinal);
    Ok((Operation::RemoveSlide { index }, true))
}

/// Take the slide at `index` out of the list.
pub(crate) fn remove_slide(parts: &mut SlideParts, index: usize) -> Result<Applied, TxnError> {
    if index >= parts.structure.slides.len() {
        return Err(TxnError::NoSuchSlide(index));
    }
    let ordinal = parts.structure.slides.remove(index);
    Ok((Operation::InsertSlide { index, ordinal }, true))
}

/// Move the slide at `from` to `to`.
pub(crate) fn move_slide(
    parts: &mut SlideParts,
    from: usize,
    to: usize,
) -> Result<Applied, TxnError> {
    let len = parts.structure.slides.len();
    if from >= len {
        return Err(TxnError::NoSuchSlide(from));
    }
    if to >= len {
        return Err(TxnError::NoSuchSlide(to));
    }
    let ordinal = parts.structure.slides.remove(from);
    parts.structure.slides.insert(to, ordinal);
    Ok((Operation::MoveSlide { from: to, to: from }, from != to))
}

/// Set a slide's `@show`.
pub(crate) fn set_show(
    parts: &mut SlideParts,
    ordinal: u32,
    show: Option<bool>,
) -> Result<Applied, TxnError> {
    let is_slide = parts
        .structure
        .entry(ordinal)
        .is_some_and(|e| e.kind == SlideKind::Slide);
    let part = parts
        .get_mut(ordinal)
        .filter(|_| is_slide)
        .ok_or(TxnError::BadPart(ordinal))?;
    let before = core::mem::replace(&mut part.show, show);
    Ok((
        Operation::SetSlideShow {
            ordinal,
            show: before,
        },
        before != show,
    ))
}
