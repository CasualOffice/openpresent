//! Alt Text (`PAR-44`, Shape Format > Alt Text and the right-click menu): a shape's
//! description for screen readers, `cNvPr/@descr`, as PowerPoint's Alt Text pane
//! writes it. OpenCalc has no alt text to copy from (owner instruction 42: checked).
//! One `SetNonVisual`, one undo step, on any shape, a group's child too. An empty
//! description removes `@descr`: absent stays absent.

use casual_slide_model::shape::ObjectId;
use casual_slide_transaction::{Operation, TxnError};

use crate::{BridgeError, Viewer};

/// The most characters of alt text kept; the rest are cut, as a pasted essay is not a
/// description. PowerPoint's pane sets no limit it states.
pub const MAX_ALT_TEXT_CHARS: usize = 4096;

impl Viewer {
    /// Shape `node` of slide `index`'s alt text, `@descr`; empty when it has none.
    ///
    /// # Errors
    ///
    /// `CS-TXN-0001` for no such shape.
    pub fn alt_text(&self, index: usize, node: u32) -> Result<String, BridgeError> {
        let part = self.slide_ordinal(index)?;
        let id = ObjectId { part, node };
        let shape = self
            .deck
            .parts()
            .get(part)
            .and_then(|p| p.node(id))
            .ok_or(TxnError::NoSuchObject(id))?;
        Ok(shape
            .non_visual
            .as_ref()
            .and_then(|n| n.descr.clone())
            .unwrap_or_default())
    }

    /// Give shape `node` of slide `index` this alt text: one undo step. Control
    /// characters but line breaks are dropped and the text cut at
    /// [`MAX_ALT_TEXT_CHARS`]; empty removes it. `false` when nothing changed.
    ///
    /// # Errors
    ///
    /// `CS-TXN-0001` for no such shape; `CS-WSM-0004` for one without `cNvPr`.
    pub fn set_alt_text(
        &mut self,
        index: usize,
        node: u32,
        text: &str,
    ) -> Result<bool, BridgeError> {
        let part = self.slide_ordinal(index)?;
        let id = ObjectId { part, node };
        let shape = self
            .deck
            .parts()
            .get(part)
            .and_then(|p| p.node(id))
            .ok_or(TxnError::NoSuchObject(id))?;
        let mut nv = shape.non_visual.clone().ok_or(BridgeError::CannotDelete)?;
        let clean: String = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(MAX_ALT_TEXT_CHARS)
            .collect();
        let clean = clean.trim();
        let descr = (!clean.is_empty()).then(|| clean.to_owned());
        if nv.descr == descr {
            return Ok(false);
        }
        nv.descr = descr;
        self.typing = None;
        let changed = self.history.apply(
            self.deck.parts_mut(),
            Operation::SetNonVisual {
                node: id,
                non_visual: Some(Box::new(nv)),
            },
        )?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../fixtures/generated/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap()
    }

    #[test]
    fn alt_text_saves_as_descr_reads_back_and_undoes() {
        let input = fixture("area-shape-tree.pptx");
        let mut v = Viewer::open(input.clone()).unwrap();
        let node = v.shapes(0).unwrap().last().unwrap().node;
        assert_eq!(v.alt_text(0, node).unwrap(), "");
        assert!(
            v.set_alt_text(0, node, "  Two ovals joined\nby an arrow \u{7}")
                .unwrap()
        );
        assert_eq!(
            v.alt_text(0, node).unwrap(),
            "Two ovals joined\nby an arrow"
        );
        let saved = v.save().unwrap();
        let reread = Viewer::open(saved.clone()).unwrap();
        let again = reread.shapes(0).unwrap().last().unwrap().node;
        assert_eq!(
            reread.alt_text(0, again).unwrap(),
            "Two ovals joined\nby an arrow"
        );
        // Emptied, `@descr` goes: the file as opened.
        assert!(v.set_alt_text(0, node, "   ").unwrap());
        assert_eq!(v.save().unwrap(), input);
        v.undo().unwrap();
        v.undo().unwrap();
        assert_eq!(v.save().unwrap(), input);
    }

    #[test]
    fn a_shape_inside_a_group_takes_alt_text() {
        let input = fixture("area-shape-tree.pptx");
        let mut v = Viewer::open(input).unwrap();
        let group = v
            .shapes(0)
            .unwrap()
            .into_iter()
            .find(|b| b.flags & crate::ShapeBox::GROUP != 0)
            .unwrap()
            .node;
        let child = v.group_shapes(0, group).unwrap()[0].node;
        assert!(v.set_alt_text(0, child, "A right arrow").unwrap());
        let reread = Viewer::open(v.save().unwrap()).unwrap();
        let g = reread
            .shapes(0)
            .unwrap()
            .into_iter()
            .find(|b| b.flags & crate::ShapeBox::GROUP != 0)
            .unwrap()
            .node;
        let c = reread.group_shapes(0, g).unwrap()[0].node;
        assert_eq!(reread.alt_text(0, c).unwrap(), "A right arrow");
    }
}
