//! Design operations (`VS-12`): a master's theme colours and fonts, the slide size, and a
//! part's background. They edit more than the slide parts, so they reach the deck through
//! [`Document`], which a host implements over whatever holds its themes and size.
//!
//! Each is its own inverse: it sets a value and returns the value it replaced, so undo
//! gives back the very model that was read and the writer then patches nothing.

use casual_slide_model::slide::SlideParts;
use casual_slide_model::theme::{BaseStyles, Theme};

use crate::{Applied, Operation, TxnError};

/// The smallest slide side, `ST_SlideSizeCoordinate`'s minimum: one inch.
/// One section of the deck as Home > Section edits it (`UX-006`): its id (a
/// brace-wrapped GUID the host supplies), its name (document text), and the part of
/// the slide it starts at. A section holds the slides from that one to the next
/// section's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionMark {
    /// `@id`.
    pub id: String,
    /// `@name`.
    pub name: String,
    /// The slide part it starts at.
    pub first: u32,
}

pub const MIN_SLIDE_SIZE: i64 = 914_400;
/// The largest slide side, `ST_SlideSizeCoordinate`'s maximum: 56 inches.
pub const MAX_SLIDE_SIZE: i64 = 51_206_400;

/// What operations edit: the slide-like parts and, for design operations, each master's
/// theme and the slide size. [`SlideParts`] alone is a document with neither, so text and
/// shape operations need nothing more, and a design operation on it is `CS-TXN-0010`.
pub trait Document {
    /// The slide-like parts.
    fn parts_mut(&mut self) -> &mut SlideParts;

    /// The theme of the master with this part ordinal, if it was read.
    fn theme_mut(&mut self, master: u32) -> Option<&mut Theme> {
        let _ = master;
        None
    }

    /// The slide size, EMU, width then height.
    fn slide_size_mut(&mut self) -> Option<&mut (i64, i64)> {
        None
    }

    /// The deck's sections (`UX-006`), as the thumbnail pane shows them.
    fn sections_mut(&mut self) -> Option<&mut Vec<SectionMark>> {
        None
    }

    /// The chart part of this name, read or added this session (`VS-11`).
    fn chart_mut(&mut self, part: &str) -> Option<&mut casual_slide_model::chart::ChartPart> {
        let _ = part;
        None
    }

    /// The deck's modern comments and authors (`UX-011`), if the host keeps them.
    fn review_mut(&mut self) -> Option<&mut crate::Review> {
        None
    }

    /// The document properties (File > Info), if this document holds them.
    fn properties_mut(
        &mut self,
    ) -> Option<&mut casual_slide_model::properties::DocumentProperties> {
        None
    }
}

impl Document for SlideParts {
    fn parts_mut(&mut self) -> &mut SlideParts {
        self
    }
}

/// A master's `themeElements`. A theme without them is refused rather than given some,
/// so an undo always gives back the very theme that was read.
fn styles<D: Document + ?Sized>(doc: &mut D, master: u32) -> Result<&mut BaseStyles, TxnError> {
    doc.theme_mut(master)
        .and_then(|t| t.elements.as_mut())
        .ok_or(TxnError::NoDesign)
}

/// Apply a design operation; `None` for any other, which the caller applies.
pub(crate) fn apply_design<D: Document + ?Sized>(
    doc: &mut D,
    op: &Operation,
) -> Result<Option<Applied>, TxnError> {
    let applied = match op {
        Operation::SetColorScheme { master, scheme } => {
            let master = *master;
            let wanted = scheme.as_deref().cloned();
            let slot = &mut styles(doc, master)?.color_scheme;
            let changed = *slot != wanted;
            let before = core::mem::replace(slot, wanted);
            (
                Operation::SetColorScheme {
                    master,
                    scheme: before.map(Box::new),
                },
                changed,
            )
        }
        Operation::SetFontScheme { master, scheme } => {
            let master = *master;
            let wanted = scheme.as_deref().cloned();
            let slot = &mut styles(doc, master)?.font_scheme;
            let changed = *slot != wanted;
            let before = core::mem::replace(slot, wanted);
            (
                Operation::SetFontScheme {
                    master,
                    scheme: before.map(Box::new),
                },
                changed,
            )
        }
        Operation::SetSlideSize { cx, cy } => {
            let (cx, cy) = (*cx, *cy);
            let fits = |v: i64| (MIN_SLIDE_SIZE..=MAX_SLIDE_SIZE).contains(&v);
            if !fits(cx) || !fits(cy) {
                return Err(TxnError::BadSlideSize { cx, cy });
            }
            let size = doc.slide_size_mut().ok_or(TxnError::NoDesign)?;
            let before = core::mem::replace(size, (cx, cy));
            (
                Operation::SetSlideSize {
                    cx: before.0,
                    cy: before.1,
                },
                before != (cx, cy),
            )
        }
        Operation::SetBackground { part, background } => {
            let part = *part;
            let id = casual_slide_model::shape::ObjectId { part, node: 0 };
            let common = doc
                .parts_mut()
                .get_mut(part)
                .and_then(|p| p.common.as_mut())
                .ok_or(TxnError::NoSuchObject(id))?;
            let wanted = background.as_deref().cloned();
            let changed = common.background != wanted;
            let before = core::mem::replace(&mut common.background, wanted);
            (
                Operation::SetBackground {
                    part,
                    background: before.map(Box::new),
                },
                changed,
            )
        }
        Operation::SetSections { sections } => {
            let slot = doc.sections_mut().ok_or(TxnError::NoDesign)?;
            let changed = slot != sections;
            let before = core::mem::replace(slot, sections.clone());
            (Operation::SetSections { sections: before }, changed)
        }
        Operation::SetDocumentProperties { properties } => {
            let slot = doc.properties_mut().ok_or(TxnError::NoDesign)?;
            let changed = **properties != *slot;
            let before = core::mem::replace(slot, (**properties).clone());
            (
                Operation::SetDocumentProperties {
                    properties: Box::new(before),
                },
                changed,
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(applied))
}

#[cfg(test)]
mod tests;
