//! The View tab's Outline View (`UX-006`): each slide's title and body placeholders,
//! as PowerPoint's outline lists them, and their text replaced from the outline as
//! one undo step.

use casual_slide_model::shape::{ObjectId, PlaceholderType};
use casual_slide_transaction::Operation;

use crate::{BridgeError, Viewer};

/// A slide's place in the outline: its part, and the nodes of its title and of its
/// body placeholder, where it has them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineSlide {
    /// The slide's part.
    pub part: u32,
    /// The title (`title` or `ctrTitle`) placeholder.
    pub title: Option<u32>,
    /// The first body placeholder (`body`, `subTitle`, `obj`, or one with no type)
    /// holding text.
    pub body: Option<u32>,
}

impl Viewer {
    /// Slide `index` in the outline.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn outline(&self, index: usize) -> Result<OutlineSlide, BridgeError> {
        let part = self.slide_ordinal(index)?;
        let mut out = OutlineSlide {
            part,
            title: None,
            body: None,
        };
        let Some(tree) = self.deck.parts().get(part).and_then(|p| p.shape_tree()) else {
            return Ok(out);
        };
        for node in tree.descendants() {
            let Some(ph) = node.placeholder() else {
                continue;
            };
            if node.text_body.is_none() {
                continue;
            }
            match ph.placeholder_type {
                Some(PlaceholderType::Title | PlaceholderType::CtrTitle) => {
                    out.title.get_or_insert(node.id.node);
                }
                None
                | Some(PlaceholderType::Body | PlaceholderType::SubTitle | PlaceholderType::Obj) => {
                    out.body.get_or_insert(node.id.node);
                }
                _ => {}
            }
        }
        Ok(out)
    }

    /// Home > Reset (`UX-006`): every placeholder on slide `index` goes back to its
    /// layout's position, size and look: its own transform and shape properties are
    /// taken away, so it inherits them again. Text and other shapes stay. One undo
    /// step; `false` when nothing was moved or restyled.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn reset_slide(&mut self, index: usize) -> Result<bool, BridgeError> {
        let part = self.slide_ordinal(index)?;
        let Some(tree) = self.deck.parts().get(part).and_then(|p| p.shape_tree()) else {
            return Ok(false);
        };
        let mut ops = Vec::new();
        for node in tree.descendants().filter(|n| n.placeholder().is_some()) {
            if node.transform.is_some() {
                ops.push(Operation::SetTransform {
                    node: node.id,
                    transform: None,
                });
            }
            if node.properties != casual_slide_model::drawing::ShapeProperties::default()
                || node.style.is_some()
            {
                ops.push(Operation::SetLook {
                    node: node.id,
                    properties: Box::new(casual_slide_model::drawing::ShapeProperties::default()),
                    style: None,
                });
            }
        }
        if ops.is_empty() {
            return Ok(false);
        }
        self.edit(Operation::Batch(ops), false)
    }

    /// Replace the whole text of `object` with `text`, a line feed starting a new
    /// paragraph, as one undo step. Formatting follows the text's first character,
    /// as typing over a selection does.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0004` for a shape with no text; the edit's own `CS-TXN` code.
    pub fn replace_text(&mut self, object: ObjectId, text: &str) -> Result<bool, BridgeError> {
        let length = self.text_length(object)?;
        let mut ops = Vec::with_capacity(2);
        if length > 0 {
            ops.push(Operation::DeleteRange {
                object,
                start: 0,
                end: length,
            });
        }
        if !text.is_empty() {
            ops.push(Operation::InsertText {
                object,
                at: 0,
                text: text.to_owned(),
            });
        }
        if ops.is_empty() {
            return Ok(false);
        }
        self.edit(Operation::Batch(ops), false)
    }
}
