//! Home > Editing: Find and Replace (`UX-006`). Every text body of every slide, in
//! slide order and then shape order, searched for plain text; Replace All is one undo
//! step. Offsets are the text engine's: a paragraph break counts one, as a line feed.

use casual_slide_model::shape::ObjectId;
use casual_slide_transaction::Operation;

use crate::{BridgeError, Viewer};

/// The most matches one search returns, so a hostile deck of repeated text cannot
/// make the page hold an unbounded list.
pub const MAX_FIND_MATCHES: usize = 10_000;

/// One place the text was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found {
    /// The slide, from 0.
    pub index: usize,
    /// The shape holding the text.
    pub object: ObjectId,
    /// Where it starts, in characters.
    pub start: usize,
    /// Where it ends.
    pub end: usize,
}

fn fold(s: &str, match_case: bool) -> Vec<char> {
    if match_case {
        s.chars().collect()
    } else {
        // One char for one, so offsets stay the text's: a char whose lower case is
        // longer keeps its own.
        s.chars()
            .map(|c| {
                let mut l = c.to_lowercase();
                match (l.next(), l.next()) {
                    (Some(one), None) => one,
                    _ => c,
                }
            })
            .collect()
    }
}

impl Viewer {
    /// Every place `query` occurs, at most [`MAX_FIND_MATCHES`]. Matches do not overlap.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0001` with no deck.
    pub fn find_text(&self, query: &str, match_case: bool) -> Result<Vec<Found>, BridgeError> {
        let needle = fold(query, match_case);
        let mut out = Vec::new();
        if needle.is_empty() {
            return Ok(out);
        }
        for index in 0..self.slide_count() {
            let part = self.slide_ordinal(index)?;
            let Some(tree) = self.deck.parts().get(part).and_then(|p| p.shape_tree()) else {
                continue;
            };
            let objects: Vec<ObjectId> = tree
                .descendants()
                .filter(|n| n.text_body.is_some())
                .map(|n| n.id)
                .collect();
            for object in objects {
                let Ok(length) = self.text_length(object) else {
                    continue;
                };
                let hay = fold(&self.text_slice(object, 0, length)?, match_case);
                let mut at = 0;
                while at + needle.len() <= hay.len() {
                    if hay.get(at..at + needle.len()) == Some(needle.as_slice()) {
                        out.push(Found {
                            index,
                            object,
                            start: at,
                            end: at + needle.len(),
                        });
                        if out.len() >= MAX_FIND_MATCHES {
                            return Ok(out);
                        }
                        at += needle.len();
                    } else {
                        at += 1;
                    }
                }
            }
        }
        Ok(out)
    }

    /// Replace every match of `query` with `with`, as one undo step; the count
    /// replaced.
    ///
    /// # Errors
    ///
    /// The edit's own `CS-TXN` code; nothing changes.
    pub fn replace_text_all(
        &mut self,
        query: &str,
        with: &str,
        match_case: bool,
    ) -> Result<usize, BridgeError> {
        let found = self.find_text(query, match_case)?;
        // From the end of each body, so earlier offsets stay true.
        let mut ops = Vec::with_capacity(found.len().saturating_mul(2));
        for f in found.iter().rev() {
            ops.push(Operation::DeleteRange {
                object: f.object,
                start: f.start,
                end: f.end,
            });
            if !with.is_empty() {
                ops.push(Operation::InsertText {
                    object: f.object,
                    at: f.start,
                    text: with.to_owned(),
                });
            }
        }
        if ops.is_empty() {
            return Ok(0);
        }
        self.edit(Operation::Batch(ops), false)?;
        Ok(found.len())
    }

    /// Replace the one match at `[start, end)` of `object` with `with`: one undo step.
    ///
    /// # Errors
    ///
    /// The edit's own `CS-TXN` code.
    pub fn replace_one(
        &mut self,
        object: ObjectId,
        start: usize,
        end: usize,
        with: &str,
    ) -> Result<bool, BridgeError> {
        let mut ops = vec![Operation::DeleteRange { object, start, end }];
        if !with.is_empty() {
            ops.push(Operation::InsertText {
                object,
                at: start,
                text: with.to_owned(),
            });
        }
        self.edit(Operation::Batch(ops), false)
    }
}
